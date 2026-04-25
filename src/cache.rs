use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Root cache directory: `$N_CACHE_DIR` or `$N_PREFIX/versions`, defaulting to `~/.n/versions`.
pub fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("N_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    crate::symlink::prefix().join("versions")
}

/// Path to the directory for a specific cached version.
pub fn version_dir(version_tag: &str) -> PathBuf {
    cache_dir().join(version_tag)
}

/// Path to the `node` binary inside a cached version directory.
pub fn node_binary(version_tag: &str) -> PathBuf {
    let dir = version_dir(version_tag);
    #[cfg(target_os = "windows")]
    return dir.join("node.exe");
    #[cfg(not(target_os = "windows"))]
    return dir.join("bin").join("node");
}

/// Returns `true` if the version is already cached on disk.
pub fn is_cached(version_tag: &str) -> bool {
    node_binary(version_tag).exists()
}

/// Returns the path to the `node` binary, or an error if it is not cached.
pub fn which(version_tag: &str) -> Result<PathBuf> {
    let path = node_binary(version_tag);
    if path.exists() {
        Ok(path)
    } else {
        anyhow::bail!("Version '{version_tag}' is not cached. Run `n install {version_tag}` first.")
    }
}

/// Remove a cached version directory.
pub fn remove(version_tag: &str) -> Result<()> {
    let dir = version_dir(version_tag);
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .with_context(|| format!("Failed to remove cached version '{version_tag}'"))?;
        println!("Removed {version_tag}");
    } else {
        println!("Version '{version_tag}' is not cached.");
    }
    Ok(())
}

/// Remove all cached versions except the currently active one.
pub fn prune() -> Result<()> {
    let active = crate::symlink::active_version();
    let dir = cache_dir();

    if !dir.exists() {
        println!("Cache directory does not exist.");
        return Ok(());
    }

    for entry in fs::read_dir(&dir).context("Failed to read cache directory")? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_default();
        if Some(&name) == active.as_ref() {
            continue;
        }
        if entry.path().is_dir() {
            fs::remove_dir_all(entry.path())
                .with_context(|| format!("Failed to remove '{name}'"))?;
            println!("Removed {name}");
        }
    }
    Ok(())
}

/// Return all locally cached version tags, newest first.
pub fn cached_versions() -> Result<Vec<String>> {
    let dir = cache_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut versions = vec![];
    for entry in fs::read_dir(&dir).context("Failed to read cache directory")? {
        let entry = entry?;
        if entry.path().is_dir() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if !name.is_empty() {
                versions.push(name);
            }
        }
    }
    // Sort newest-first (lexicographic is fine for v-prefixed semver)
    versions.sort_by(|a, b| b.cmp(a));
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_cache<F: FnOnce(&std::path::Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_CACHE_DIR", dir.path());
        f(dir.path());
        std::env::remove_var("N_CACHE_DIR");
    }

    #[test]
    fn cache_dir_respects_env_var() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_CACHE_DIR", dir.path());
        let result = cache_dir();
        std::env::remove_var("N_CACHE_DIR");
        assert_eq!(result, dir.path());
    }

    #[test]
    fn version_dir_is_under_cache_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_CACHE_DIR", dir.path());
        let vdir = version_dir("v20.11.0");
        std::env::remove_var("N_CACHE_DIR");
        assert_eq!(vdir, dir.path().join("v20.11.0"));
    }

    #[test]
    fn node_binary_is_inside_version_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_CACHE_DIR", dir.path());
        let bin = node_binary("v20.11.0");
        std::env::remove_var("N_CACHE_DIR");
        assert!(bin.starts_with(dir.path()));
        let name = bin.file_name().unwrap().to_string_lossy();
        assert!(name == "node" || name == "node.exe");
    }

    #[test]
    fn is_cached_returns_false_when_missing() {
        with_temp_cache(|_| {
            assert!(!is_cached("v99.0.0"));
        });
    }

    #[test]
    fn is_cached_returns_true_when_binary_exists() {
        with_temp_cache(|base| {
            let vdir = base.join("v20.11.0").join("bin");
            fs::create_dir_all(&vdir).unwrap();
            fs::write(vdir.join("node"), b"fake").unwrap();
            assert!(is_cached("v20.11.0"));
        });
    }

    #[test]
    fn which_errors_when_not_cached() {
        with_temp_cache(|_| {
            assert!(which("v99.0.0").is_err());
        });
    }

    #[test]
    fn which_returns_path_when_cached() {
        with_temp_cache(|base| {
            let vdir = base.join("v20.11.0").join("bin");
            fs::create_dir_all(&vdir).unwrap();
            fs::write(vdir.join("node"), b"fake").unwrap();
            assert!(which("v20.11.0").is_ok());
        });
    }

    #[test]
    fn remove_deletes_version_dir() {
        with_temp_cache(|base| {
            let vdir = base.join("v20.11.0");
            fs::create_dir_all(&vdir).unwrap();
            remove("v20.11.0").unwrap();
            assert!(!vdir.exists());
        });
    }

    #[test]
    fn remove_is_ok_when_not_cached() {
        with_temp_cache(|_| {
            assert!(remove("v99.0.0").is_ok());
        });
    }

    #[test]
    fn cached_versions_returns_empty_when_dir_missing() {
        with_temp_cache(|base| {
            fs::remove_dir_all(base).unwrap();
            assert_eq!(cached_versions().unwrap(), Vec::<String>::new());
        });
    }

    #[test]
    fn cached_versions_returns_sorted_desc() {
        with_temp_cache(|base| {
            fs::create_dir_all(base.join("v18.0.0")).unwrap();
            fs::create_dir_all(base.join("v20.11.0")).unwrap();
            fs::create_dir_all(base.join("v22.0.0")).unwrap();
            let versions = cached_versions().unwrap();
            assert_eq!(versions[0], "v22.0.0");
        });
    }

    #[test]
    fn prune_removes_inactive_versions() {
        let _guard = ENV_LOCK.lock().unwrap();
        let cache = tempfile::tempdir().expect("tempdir");
        let prefix = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_CACHE_DIR", cache.path());
        std::env::set_var("N_PREFIX", prefix.path());

        // Active version marker
        fs::write(prefix.path().join(".active"), "v20.11.0").unwrap();

        // Create two cached versions
        fs::create_dir_all(cache.path().join("v20.11.0")).unwrap();
        fs::create_dir_all(cache.path().join("v18.0.0")).unwrap();

        prune().unwrap();

        assert!(cache.path().join("v20.11.0").exists());
        assert!(!cache.path().join("v18.0.0").exists());

        std::env::remove_var("N_CACHE_DIR");
        std::env::remove_var("N_PREFIX");
    }
}
