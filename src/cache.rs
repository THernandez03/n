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
        anyhow::bail!("Version '{version_tag}' is not cached. Run `n {version_tag}` to install it.")
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
/// Remove all cached versions except the currently active one.
/// When `force` is `true`, all versions including the active one are removed.
pub fn prune(force: bool) -> Result<()> {
    let active = crate::symlink::active_version();
    let dir = cache_dir();

    if !dir.exists() {
        println!("Cache directory does not exist.");
        return Ok(());
    }

    for entry in fs::read_dir(&dir).context("Failed to read cache directory")? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_default();
        if !force && Some(&name) == active.as_ref() {
            println!("Skipped {name} (active — use --force to remove)");
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

/// Returns `true` if `a` is a prefix of `b` or `b` is a prefix of `a`.
/// Used for fuzzy SHA matching between stored (short) and user-provided (long) SHAs.
pub fn sha_matches(a: &str, b: &str) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

/// Returns the SHA portion of a cache key (the part after `+`), if any.
pub fn cache_key_sha(key: &str) -> Option<&str> {
    key.split_once('+').map(|(_, sha)| sha)
}

/// Find a cached version matching the given version prefix.
///
/// A match occurs when the cache-directory name equals `prefix` exactly, starts
/// with `"{prefix}+"` (exact version with SHA), or starts with `"{prefix}."`
/// (partial version, e.g. `"22"` matches `"22.11.0"`).
/// If multiple entries match, the most recently modified is returned.
pub fn find_by_version_prefix(prefix: &str) -> Option<String> {
    let dir = cache_dir();
    if !dir.exists() {
        return None;
    }
    let prefix_plus = format!("{prefix}+");
    let prefix_dot = format!("{prefix}.");
    let mut best: Option<(std::time::SystemTime, String)> = None;
    let Ok(entries) = fs::read_dir(&dir) else {
        return None;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if (name == prefix || name.starts_with(&prefix_plus) || name.starts_with(&prefix_dot))
            && node_binary(&name).exists()
        {
            let mtime = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let is_newer = best.as_ref().map_or(true, |(t, _)| mtime > *t);
            if is_newer {
                best = Some((mtime, name));
            }
        }
    }
    best.map(|(_, name)| name)
}

/// Find a cached version whose SHA component fuzzy-matches the given SHA.
///
/// The SHA component is the part after `+` in the cache key.
/// If multiple entries match, the most recently modified is returned.
pub fn find_by_sha(sha: &str) -> Option<String> {
    let dir = cache_dir();
    if !dir.exists() {
        return None;
    }
    let mut best: Option<(std::time::SystemTime, String)> = None;
    let Ok(entries) = fs::read_dir(&dir) else {
        return None;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if let Some(cached_sha) = cache_key_sha(&name) {
            if sha_matches(cached_sha, sha) && node_binary(&name).exists() {
                let mtime = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let is_newer = best.as_ref().map_or(true, |(t, _)| mtime > *t);
                if is_newer {
                    best = Some((mtime, name));
                }
            }
        }
    }
    best.map(|(_, name)| name)
}

/// Rename a cached version directory from `old_key` to `new_key`.
/// No-op if `old_key` does not exist or `new_key` already exists.
pub fn rename_version(old_key: &str, new_key: &str) -> Result<()> {
    let old_dir = version_dir(old_key);
    let new_dir = version_dir(new_key);
    if old_dir.exists() && !new_dir.exists() {
        fs::rename(&old_dir, &new_dir).with_context(|| {
            format!("Failed to rename cache entry '{old_key}' \u{2192} '{new_key}'")
        })?;
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

        prune(false).unwrap();

        assert!(cache.path().join("v20.11.0").exists());
        assert!(!cache.path().join("v18.0.0").exists());

        std::env::remove_var("N_CACHE_DIR");
        std::env::remove_var("N_PREFIX");
    }

    #[test]
    fn prune_force_removes_all_versions() {
        let _guard = ENV_LOCK.lock().unwrap();
        let cache = tempfile::tempdir().expect("tempdir");
        let prefix = tempfile::tempdir().expect("tempdir");
        std::env::set_var("N_CACHE_DIR", cache.path());
        std::env::set_var("N_PREFIX", prefix.path());

        fs::write(prefix.path().join(".active"), "v20.11.0").unwrap();
        fs::create_dir_all(cache.path().join("v20.11.0")).unwrap();
        fs::create_dir_all(cache.path().join("v18.0.0")).unwrap();

        prune(true).unwrap();

        assert!(
            !cache.path().join("v20.11.0").exists(),
            "--force should remove active"
        );
        assert!(
            !cache.path().join("v18.0.0").exists(),
            "--force should remove inactive"
        );

        std::env::remove_var("N_CACHE_DIR");
        std::env::remove_var("N_PREFIX");
    }

    fn make_cached_node(tag: &str) {
        let path = node_binary(tag);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"fake").unwrap();
    }

    // ── sha_matches ───────────────────────────────────────────────────

    #[test]
    fn sha_matches_identical() {
        assert!(sha_matches("abc1234def", "abc1234def"));
    }

    #[test]
    fn sha_matches_a_prefix_of_b() {
        assert!(sha_matches("abc1234", "abc1234def5678"));
    }

    #[test]
    fn sha_matches_b_prefix_of_a() {
        assert!(sha_matches("abc1234def5678", "abc1234"));
    }

    #[test]
    fn sha_matches_unrelated_returns_false() {
        assert!(!sha_matches("abc1234", "def5678"));
    }

    // ── cache_key_sha ─────────────────────────────────────────────────

    #[test]
    fn cache_key_sha_present() {
        assert_eq!(cache_key_sha("22.11.0+abc1234def"), Some("abc1234def"));
    }

    #[test]
    fn cache_key_sha_absent() {
        assert!(cache_key_sha("22.11.0").is_none());
    }

    // ── find_by_version_prefix ────────────────────────────────────────

    #[test]
    fn find_by_version_prefix_exact() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0");
            assert_eq!(
                find_by_version_prefix("22.11.0"),
                Some("22.11.0".to_string())
            );
        });
    }

    #[test]
    fn find_by_version_prefix_with_sha_suffix() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0+abc1234def");
            assert_eq!(
                find_by_version_prefix("22.11.0"),
                Some("22.11.0+abc1234def".to_string())
            );
        });
    }

    #[test]
    fn find_by_version_prefix_dot_match() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0");
            assert_eq!(find_by_version_prefix("22.11"), Some("22.11.0".to_string()));
        });
    }

    #[test]
    fn find_by_version_prefix_no_match() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0");
            assert!(find_by_version_prefix("20.11.0").is_none());
        });
    }

    #[test]
    fn find_by_version_prefix_requires_binary() {
        with_temp_cache(|base| {
            fs::create_dir_all(base.join("22.11.0")).unwrap();
            assert!(find_by_version_prefix("22.11.0").is_none());
        });
    }

    // ── find_by_sha ───────────────────────────────────────────────────

    #[test]
    fn find_by_sha_exact() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0+abc1234def");
            assert_eq!(
                find_by_sha("abc1234def"),
                Some("22.11.0+abc1234def".to_string())
            );
        });
    }

    #[test]
    fn find_by_sha_input_prefix_of_stored() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0+abc1234def5678");
            assert_eq!(
                find_by_sha("abc1234d"),
                Some("22.11.0+abc1234def5678".to_string())
            );
        });
    }

    #[test]
    fn find_by_sha_stored_prefix_of_input() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0+abc1234d");
            assert_eq!(
                find_by_sha("abc1234def5678"),
                Some("22.11.0+abc1234d".to_string())
            );
        });
    }

    #[test]
    fn find_by_sha_no_match() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0+abc1234def");
            assert!(find_by_sha("xyz99999").is_none());
        });
    }

    #[test]
    fn find_by_sha_ignores_entry_without_sha() {
        with_temp_cache(|_| {
            make_cached_node("22.11.0");
            assert!(find_by_sha("22110").is_none());
        });
    }

    // ── rename_version ────────────────────────────────────────────────

    #[test]
    fn rename_version_moves_dir() {
        with_temp_cache(|base| {
            fs::create_dir_all(base.join("22.11.0")).unwrap();
            rename_version("22.11.0", "22.11.0+abc1234def").unwrap();
            assert!(!base.join("22.11.0").exists());
            assert!(base.join("22.11.0+abc1234def").exists());
        });
    }

    #[test]
    fn rename_version_noop_when_old_missing() {
        with_temp_cache(|_| {
            assert!(rename_version("nonexistent", "also-nonexistent").is_ok());
        });
    }

    #[test]
    fn rename_version_noop_when_new_exists() {
        with_temp_cache(|base| {
            fs::create_dir_all(base.join("22.11.0")).unwrap();
            fs::create_dir_all(base.join("22.11.0+abc1234def")).unwrap();
            rename_version("22.11.0", "22.11.0+abc1234def").unwrap();
            assert!(base.join("22.11.0").exists());
            assert!(base.join("22.11.0+abc1234def").exists());
        });
    }
}
