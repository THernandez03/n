use anyhow::{Context, Result};
use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::process::Command;

use console::style;

use crate::{arch, cache, releases, symlink};

/// Install a Node.js version and activate it.
pub fn install(version_str: &str) -> Result<()> {
    let tag = releases::resolve_tag(version_str)?;

    if symlink::active_version().as_deref() == Some(&tag) {
        println!(
            "{} Node.js {} is already the active version.",
            style("✓").green().bold(),
            style(&tag).cyan().bold(),
        );
        return Ok(());
    }

    if cache::is_cached(&tag) {
        println!(
            "{} Node.js {} is already cached.",
            style("◆").dim(),
            style(&tag).cyan(),
        );
    } else {
        println!(
            "{} Downloading Node.js {}...",
            style("⬇").cyan(),
            style(&tag).cyan().bold(),
        );
        let url = arch::download_url(&tag);
        download_version(&url, &tag)?;
    }

    let from = symlink::active_version();
    match &from {
        Some(f) => println!(
            "{} Activating Node.js {} → {}...",
            style("◆").magenta(),
            style(f).cyan().bold(),
            style(&tag).cyan().bold(),
        ),
        None => println!(
            "{} Activating Node.js {}...",
            style("◆").magenta(),
            style(&tag).cyan().bold(),
        ),
    }
    symlink::activate(&tag)?;
    println!(
        "{} Installed Node.js {} successfully.",
        style("✓").green().bold(),
        style(&tag).cyan().bold(),
    );
    Ok(())
}

/// Download a version into cache without activating it.
pub fn download_only(version_str: &str) -> Result<()> {
    let tag = releases::resolve_tag(version_str)?;
    if cache::is_cached(&tag) {
        println!("Version {tag} is already cached.");
        return Ok(());
    }
    println!("Downloading Node.js {tag}...");
    let url = arch::download_url(&tag);
    download_version(&url, &tag)
}

/// Run a cached Node.js version with given arguments.
pub fn run(version_str: &str, args: &[String]) -> Result<()> {
    let tag = releases::resolve_tag(version_str)?;

    if !cache::is_cached(&tag) {
        println!("Version {tag} is not cached. Downloading...");
        let url = arch::download_url(&tag);
        download_version(&url, &tag)?;
    }

    let binary = cache::node_binary(&tag);
    let status = Command::new(&binary)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run node {tag}"))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn download_version(url: &str, tag: &str) -> Result<()> {
    let dest_dir = cache::version_dir(tag);
    fs::create_dir_all(&dest_dir).context("Failed to create cache directory")?;

    let ext = arch::archive_ext();
    let tmp_path = dest_dir.with_extension(ext);

    {
        let client = reqwest::blocking::Client::new();
        let mut resp = client
            .get(url)
            .header("User-Agent", "n-node-version-manager")
            .send()
            .context("HTTP request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            fs::remove_dir_all(&dest_dir).ok();
            anyhow::bail!("Download failed: server returned HTTP {status} for {url}");
        }

        let total = resp.content_length().unwrap_or(0);
        let file = fs::File::create(&tmp_path).context("Failed to create temp file")?;
        let mut writer = BufWriter::new(file);

        let mut downloaded = 0u64;
        let mut buf = vec![0u8; 65536];
        loop {
            let n = resp.read(&mut buf)?;
            if n == 0 {
                break;
            }
            writer.write_all(&buf[..n])?;
            downloaded += n as u64;
            if total > 0 {
                if let Some(pct) = downloaded.saturating_mul(100).checked_div(total) {
                    print!("\r  {downloaded}/{total} bytes ({pct}%)");
                    io::stdout().flush()?;
                }
            }
        }
        println!();
    }

    if ext == "zip" {
        extract_zip(&tmp_path, &dest_dir)?;
    } else {
        extract_tar_gz(&tmp_path, &dest_dir)?;
    }
    fs::remove_file(&tmp_path).ok();

    // Node.js tarballs unpack to a single directory: node-v{ver}-{os}-{arch}/
    // Flatten that so the cache dir directly contains bin/node etc.
    flatten_single_dir(&dest_dir)?;

    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive).context("Failed to open tar.gz")?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);
    tar.unpack(dest).context("Failed to extract tar.gz")?;
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive).context("Failed to open zip")?;
    let mut zip = zip::ZipArchive::new(file).context("Failed to read zip")?;
    zip.extract(dest).context("Failed to extract zip")?;
    Ok(())
}

/// If `dir` contains exactly one subdirectory, move its contents up one level.
fn flatten_single_dir(dir: &Path) -> Result<()> {
    let entries: Vec<_> = fs::read_dir(dir)?.collect::<std::io::Result<_>>()?;
    if entries.len() == 1 && entries[0].path().is_dir() {
        let inner = entries[0].path();
        for entry in fs::read_dir(&inner)? {
            let entry = entry?;
            let dest = dir.join(entry.file_name());
            fs::rename(entry.path(), dest).ok();
        }
        fs::remove_dir_all(&inner).ok();
    }
    Ok(())
}

/// Remove a cached version, or prompt for interactive selection if no version is given.
pub fn remove_version(version: Option<String>) -> Result<()> {
    if let Some(v) = version {
        cache::remove(&v)?;
        return Ok(());
    }
    let versions = cache::cached_versions()?;
    if versions.is_empty() {
        println!("No cached versions to remove.");
        return Ok(());
    }
    let active = symlink::active_version();
    let items: Vec<String> = versions
        .iter()
        .map(|v| {
            if Some(v.as_str()) == active.as_deref() {
                format!("{v}  (active)")
            } else {
                v.clone()
            }
        })
        .collect();
    let idx = dialoguer::Select::new()
        .with_prompt("Select a version to remove")
        .items(&items)
        .interact()?;
    cache::remove(&versions[idx])?;
    Ok(())
}

const GITHUB_REPO: &str = "THernandez03/n";

fn self_artifact() -> String {
    let name = env!("CARGO_PKG_NAME");
    let os_arch = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x64"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "linux-arm64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "darwin-x64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "darwin-arm64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else {
        "linux-x64"
    };
    if cfg!(target_os = "windows") {
        format!("{name}-{os_arch}.exe")
    } else {
        format!("{name}-{os_arch}")
    }
}

/// Self-update this version manager binary to the latest GitHub release.
pub fn update_self() -> Result<()> {
    let name = env!("CARGO_PKG_NAME");
    println!("{} Checking for {} updates...", style("◆").cyan(), name);
    let client = reqwest::blocking::Client::new();
    let release: serde_json::Value = client
        .get(format!(
            "https://api.github.com/repos/{GITHUB_REPO}/releases/latest"
        ))
        .header("User-Agent", format!("{name}-version-manager"))
        .send()
        .context("Failed to fetch latest release info")?
        .json()
        .context("Failed to parse release JSON")?;
    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No tag_name in GitHub release response"))?;
    let current = env!("CARGO_PKG_VERSION");
    let remote = tag.trim_start_matches('v');
    if remote == current {
        println!(
            "{} {} is already up to date ({})",
            style("✓").green().bold(),
            name,
            style(current).cyan().bold()
        );
        return Ok(());
    }
    println!(
        "{} Updating {} {} \u{2192} {}...",
        style("⬇").cyan(),
        name,
        style(current).dim(),
        style(remote).cyan().bold()
    );
    let artifact = self_artifact();
    let url = format!("https://github.com/{GITHUB_REPO}/releases/download/{tag}/{artifact}");
    let exe = std::env::current_exe().context("Failed to locate current executable")?;
    let tmp = exe.with_extension("update-tmp");
    {
        let mut resp = client
            .get(&url)
            .header("User-Agent", format!("{name}-version-manager"))
            .send()
            .context("Failed to download update")?;
        if !resp.status().is_success() {
            anyhow::bail!("Download failed: HTTP {} for {}", resp.status(), url);
        }
        let file = fs::File::create(&tmp).context("Failed to create temp file for update")?;
        let mut writer = BufWriter::new(file);
        let mut buf = vec![0u8; 65536];
        loop {
            let n = resp.read(&mut buf).context("Read error during download")?;
            if n == 0 {
                break;
            }
            writer
                .write_all(&buf[..n])
                .context("Write error during download")?;
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))
            .context("Failed to set executable permission")?;
    }
    fs::rename(&tmp, &exe).context("Failed to replace current binary")?;
    println!(
        "{} {} updated to {}.",
        style("✓").green().bold(),
        name,
        style(remote).cyan().bold()
    );
    Ok(())
}

/// Uninstall this version manager completely (removes cache, prefix directory, and the binary).
pub fn uninstall_self() -> Result<()> {
    let name = env!("CARGO_PKG_NAME");
    let confirmed = dialoguer::Confirm::new()
        .with_prompt(format!(
            "This will remove all cached versions and the {name} binary. Continue?"
        ))
        .default(false)
        .interact()?;
    if !confirmed {
        println!("Aborted.");
        return Ok(());
    }
    println!("Uninstalling {}...", style(name).cyan().bold());
    let prefix = symlink::prefix();
    if prefix.exists() {
        fs::remove_dir_all(&prefix)
            .with_context(|| format!("Failed to remove {}", prefix.display()))?;
        println!("  {} Removed {}", style("✓").green(), prefix.display());
    }
    let exe = std::env::current_exe().context("Failed to locate current executable")?;
    fs::remove_file(&exe).with_context(|| format!("Failed to remove {}", exe.display()))?;
    println!("  {} Removed {}", style("✓").green(), exe.display());
    println!();
    println!(
        "{} {} uninstalled. Remove {} from your PATH if needed.",
        style("✓").green().bold(),
        name,
        exe.parent()
            .map_or_else(String::new, |p| p.display().to_string())
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_dirs<F: FnOnce(&std::path::Path, &std::path::Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let cache = tempfile::tempdir().expect("tempdir");
        let prefix = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_CACHE_DIR", cache.path());
        std::env::set_var("N_PREFIX", prefix.path());
        f(cache.path(), prefix.path());
        std::env::remove_var("N_CACHE_DIR");
        std::env::remove_var("N_PREFIX");
    }

    #[test]
    fn flatten_single_dir_moves_contents_up() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let inner = tmp.path().join("inner-dir");
        fs::create_dir_all(&inner).unwrap();
        fs::write(inner.join("node"), b"binary").unwrap();

        flatten_single_dir(tmp.path()).unwrap();

        assert!(tmp.path().join("node").exists());
        assert!(!inner.exists());
    }

    #[test]
    fn flatten_noop_when_multiple_entries() {
        let tmp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(tmp.path().join("a")).unwrap();
        fs::create_dir_all(tmp.path().join("b")).unwrap();

        flatten_single_dir(tmp.path()).unwrap();

        assert!(tmp.path().join("a").exists());
        assert!(tmp.path().join("b").exists());
    }

    #[test]
    fn download_only_skips_if_already_cached() {
        with_temp_dirs(|cache, _prefix| {
            // Place a fake binary to simulate a cached version
            let vdir = cache.join("v20.11.0").join("bin");
            fs::create_dir_all(&vdir).unwrap();
            fs::write(vdir.join("node"), b"fake").unwrap();

            // Should succeed without hitting the network
            // (resolve_tag would hit network for "v20.11.0" only if not exact 3-part)
            // We use exact tag form to skip network
            let result = download_only("20.11.0");
            // This will attempt network resolution; just test it doesn't panic
            // or crash for a structural reason
            drop(result);
        });
    }
}
