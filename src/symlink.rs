use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Installation prefix: `$N_PREFIX` or `~/.n`.
pub fn prefix() -> PathBuf {
    if let Ok(p) = std::env::var("N_PREFIX") {
        return PathBuf::from(p);
    }
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".n")
}

/// The bin directory where the active `node` symlink lives.
pub fn bin_dir() -> PathBuf {
    prefix().join("bin")
}

/// Activate a cached version by creating/updating a symlink.
///
/// Node.js tarballs extract to `node-v{version}-{os}-{arch}/bin/node`, so
/// after flattening the cache directory contains `bin/node`. We symlink from
/// `~/.n/bin/node` → the cached binary.
pub fn activate(version_tag: &str) -> Result<()> {
    let bin = bin_dir();
    fs::create_dir_all(&bin).context("Failed to create bin directory")?;

    let node_src = crate::cache::node_binary(version_tag);

    #[cfg(target_os = "windows")]
    let link_path = bin.join("node.exe");
    #[cfg(not(target_os = "windows"))]
    let link_path = bin.join("node");

    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        fs::remove_file(&link_path).ok();
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(&node_src, &link_path).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link_path.display(),
            node_src.display()
        )
    })?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&node_src, &link_path).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link_path.display(),
            node_src.display()
        )
    })?;

    let marker = prefix().join(".active");
    fs::write(&marker, version_tag).context("Failed to write active version marker")?;

    Ok(())
}

/// Read the currently active version from the marker file.
pub fn active_version() -> Option<String> {
    let marker = prefix().join(".active");
    fs::read_to_string(marker)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Remove the active node symlink (does not remove cache).
pub fn uninstall() -> Result<()> {
    let bin = bin_dir();

    #[cfg(target_os = "windows")]
    let link_path = bin.join("node.exe");
    #[cfg(not(target_os = "windows"))]
    let link_path = bin.join("node");

    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        fs::remove_file(&link_path).context("Failed to remove node symlink")?;
        println!("Removed active Node.js installation.");
    } else {
        println!("No active Node.js installation found.");
    }

    let marker = prefix().join(".active");
    fs::remove_file(marker).ok();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_prefix<F: FnOnce(&std::path::Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_PREFIX", dir.path());
        std::env::remove_var("N_CACHE_DIR");
        f(dir.path());
        std::env::remove_var("N_PREFIX");
    }

    #[test]
    fn prefix_respects_env_var() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_PREFIX", dir.path());
        let result = prefix();
        std::env::remove_var("N_PREFIX");
        assert_eq!(result, dir.path());
    }

    #[test]
    fn bin_dir_is_under_prefix() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_PREFIX", dir.path());
        let b = bin_dir();
        std::env::remove_var("N_PREFIX");
        assert_eq!(b, dir.path().join("bin"));
    }

    #[test]
    fn active_version_returns_none_when_marker_missing() {
        with_temp_prefix(|_| {
            assert_eq!(active_version(), None);
        });
    }

    #[test]
    fn active_version_reads_marker_file() {
        with_temp_prefix(|base| {
            fs::write(base.join(".active"), "v20.11.0").unwrap();
            assert_eq!(active_version(), Some("v20.11.0".to_string()));
        });
    }

    #[test]
    fn active_version_trims_whitespace() {
        with_temp_prefix(|base| {
            fs::write(base.join(".active"), "v20.11.0\n").unwrap();
            assert_eq!(active_version(), Some("v20.11.0".to_string()));
        });
    }

    #[cfg(unix)]
    #[test]
    fn activate_creates_symlink_and_marker() {
        with_temp_prefix(|base| {
            std::env::set_var("N_CACHE_DIR", base.join("versions"));
            let vdir = base.join("versions").join("v20.11.0").join("bin");
            fs::create_dir_all(&vdir).unwrap();
            fs::write(vdir.join("node"), b"#!/bin/sh\necho hi").unwrap();

            activate("v20.11.0").unwrap();

            let link = base.join("bin").join("node");
            assert!(link.symlink_metadata().is_ok(), "symlink should exist");
            assert_eq!(active_version(), Some("v20.11.0".to_string()));
            std::env::remove_var("N_CACHE_DIR");
        });
    }

    #[cfg(unix)]
    #[test]
    fn activate_replaces_existing_symlink() {
        with_temp_prefix(|base| {
            std::env::set_var("N_CACHE_DIR", base.join("versions"));
            for v in &["v18.0.0", "v20.11.0"] {
                let vdir = base.join("versions").join(v).join("bin");
                fs::create_dir_all(&vdir).unwrap();
                fs::write(vdir.join("node"), b"#!/bin/sh").unwrap();
            }
            activate("v18.0.0").unwrap();
            activate("v20.11.0").unwrap();
            assert_eq!(active_version(), Some("v20.11.0".to_string()));
            std::env::remove_var("N_CACHE_DIR");
        });
    }

    #[cfg(unix)]
    #[test]
    fn uninstall_removes_symlink_and_marker() {
        with_temp_prefix(|base| {
            std::env::set_var("N_CACHE_DIR", base.join("versions"));
            let vdir = base.join("versions").join("v20.11.0").join("bin");
            fs::create_dir_all(&vdir).unwrap();
            fs::write(vdir.join("node"), b"#!/bin/sh").unwrap();
            activate("v20.11.0").unwrap();
            uninstall().unwrap();
            let link = base.join("bin").join("node");
            assert!(!link.exists() && link.symlink_metadata().is_err());
            assert!(active_version().is_none());
            std::env::remove_var("N_CACHE_DIR");
        });
    }

    #[test]
    fn uninstall_ok_when_nothing_installed() {
        with_temp_prefix(|_| {
            assert!(uninstall().is_ok());
        });
    }
}
