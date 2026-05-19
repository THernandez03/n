use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::{cache, symlink};

/// Print locally cached versions.
pub fn list_local() -> Result<()> {
    let versions = cache::cached_versions()?;
    let active = symlink::active_version();

    if versions.is_empty() {
        println!("No cached Node.js versions found.");
        println!("Run `n install <version>` to install one.");
        return Ok(());
    }

    println!("Cached Node.js versions:");
    for v in &versions {
        let marker = if Some(v) == active.as_ref() {
            " (active)"
        } else {
            ""
        };
        println!("  {v}{marker}");
    }

    Ok(())
}

/// Interactive version picker using arrow keys.
pub fn interactive_picker() -> Result<()> {
    let versions = cache::cached_versions()?;

    if versions.is_empty() {
        println!("No cached Node.js versions found.");
        println!("Run `n install <version>` or `n ls-remote` to get started.");
        return Ok(());
    }

    let active = symlink::active_version();
    let items: Vec<String> = versions
        .iter()
        .map(|v| {
            if Some(v) == active.as_ref() {
                format!("{v} *")
            } else {
                v.clone()
            }
        })
        .collect();

    let default = versions
        .iter()
        .position(|v| Some(v) == active.as_ref())
        .unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a Node.js version (arrow keys, Enter to install, q to quit)")
        .default(default)
        .items(&items)
        .interact_opt()?
        .unwrap_or(usize::MAX);

    if selection == usize::MAX {
        return Ok(());
    }

    let chosen = &versions[selection];
    crate::install::install(chosen)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn format_versions(versions: &[String], active: Option<&str>) -> Vec<String> {
        versions
            .iter()
            .map(|v| {
                let marker = if Some(v.as_str()) == active {
                    " (active)"
                } else {
                    ""
                };
                format!("  {v}{marker}")
            })
            .collect()
    }

    fn format_items(versions: &[String], active: Option<&str>) -> Vec<String> {
        versions
            .iter()
            .map(|v| {
                if Some(v.as_str()) == active {
                    format!("{v} *")
                } else {
                    v.clone()
                }
            })
            .collect()
    }

    fn with_temp_env<F: FnOnce()>(cache_dir: &std::path::Path, prefix_dir: &std::path::Path, f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("N_CACHE_DIR", cache_dir);
        std::env::set_var("N_PREFIX", prefix_dir);
        f();
        std::env::remove_var("N_CACHE_DIR");
        std::env::remove_var("N_PREFIX");
    }

    #[test]
    fn format_versions_no_active() {
        let versions = ["v20.11.0".to_string(), "v18.0.0".to_string()];
        let lines = format_versions(&versions, None);
        assert_eq!(lines, vec!["  v20.11.0", "  v18.0.0"]);
    }

    #[test]
    fn format_versions_marks_active() {
        let versions = ["v20.11.0".to_string(), "v18.0.0".to_string()];
        let lines = format_versions(&versions, Some("v20.11.0"));
        assert_eq!(lines[0], "  v20.11.0 (active)");
        assert_eq!(lines[1], "  v18.0.0");
    }

    #[test]
    fn format_items_no_active() {
        let versions = ["v20.11.0".to_string()];
        let items = format_items(&versions, None);
        assert_eq!(items, vec!["v20.11.0"]);
    }

    #[test]
    fn format_items_marks_active_with_star() {
        let versions = ["v20.11.0".to_string(), "v18.0.0".to_string()];
        let items = format_items(&versions, Some("v20.11.0"));
        assert_eq!(items[0], "v20.11.0 *");
        assert_eq!(items[1], "v18.0.0");
    }

    #[test]
    fn default_index_for_active_version() {
        let versions = [
            "v22.0.0".to_string(),
            "v20.11.0".to_string(),
            "v18.0.0".to_string(),
        ];
        let active = Some("v20.11.0".to_string());
        let idx = versions
            .iter()
            .position(|v| Some(v) == active.as_ref())
            .unwrap_or(0);
        assert_eq!(idx, 1);
    }

    #[test]
    fn list_local_works_with_empty_cache() {
        let cache_dir = tempfile::tempdir().expect("tempdir");
        let prefix_dir = tempfile::tempdir().expect("tempdir");
        fs::remove_dir_all(cache_dir.path()).unwrap();
        with_temp_env(cache_dir.path(), prefix_dir.path(), || {
            assert!(super::list_local().is_ok());
        });
    }

    #[test]
    fn list_local_works_with_versions() {
        let cache_dir = tempfile::tempdir().expect("tempdir");
        let prefix_dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(cache_dir.path().join("v20.11.0")).unwrap();
        with_temp_env(cache_dir.path(), prefix_dir.path(), || {
            assert!(super::list_local().is_ok());
        });
    }
}
