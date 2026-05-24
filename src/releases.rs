use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

const INDEX_URL: &str = "https://nodejs.org/dist/index.json";

#[derive(Debug, Deserialize, Clone)]
pub struct NodeRelease {
    pub version: String,
    pub lts: serde_json::Value, // false or "Iron" / "Hydrogen" etc.
}

impl NodeRelease {
    /// Returns the LTS codename if this is an LTS release, or `None`.
    #[must_use]
    pub fn lts_name(&self) -> Option<&str> {
        self.lts.as_str()
    }

    /// Returns `true` if this is an LTS release.
    #[must_use]
    pub fn is_lts(&self) -> bool {
        self.lts_name().is_some()
    }
}

/// Fetch the full release index from nodejs.org.
pub fn fetch_releases() -> Result<Vec<NodeRelease>> {
    let client = Client::new();
    let releases: Vec<NodeRelease> = client
        .get(INDEX_URL)
        .header("User-Agent", "n-node-version-manager")
        .send()
        .context("Failed to fetch Node.js release index")?
        .json()
        .context("Failed to parse Node.js release index JSON")?;
    Ok(releases)
}

/// Print recent Node.js releases (latest 20).
pub fn list_remote() -> Result<()> {
    let releases = fetch_releases()?;
    println!("Available Node.js versions (recent 20):");
    for r in releases.iter().take(20) {
        let lts = r
            .lts_name()
            .map_or(String::new(), |n| format!(" (LTS: {n})"));
        println!("  {}{}", r.version, lts);
    }
    Ok(())
}

/// Resolve a user-supplied version string to an exact version tag (e.g. `22.11.0`).
///
/// Aliases supported:
/// - `"lts"` / `"stable"` / `"current"` → latest LTS release
/// - `"latest"` / `"canary"` / `"next"` / `"nightly"` / `"edge"` / `""` → newest release (may not be LTS)
/// - `"20"` / `"20.x"` → latest release in major 20
/// - `"20.11"` / `"20.11.x"` → latest patch in 20.11
/// - `"20.11.0"` → exact version, no network lookup needed
pub fn resolve_tag(version_str: &str) -> Result<String> {
    let v = version_str.trim();

    // Reject v-prefixed version strings
    if v.starts_with('v') && v[1..].starts_with(|c: char| c.is_ascii_digit()) {
        anyhow::bail!("No Node.js release found matching '{v}'");
    }

    // Exact version — three-part semver (no leading v)
    if v.split('.').count() >= 3 && v.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Ok(v.to_string());
    }

    // Aliases that need the release list
    let releases = fetch_releases()?;
    resolve_from(v, &releases)
}

/// Pure resolver that operates on a pre-fetched list (used by tests).
pub fn resolve_from(version_str: &str, releases: &[NodeRelease]) -> Result<String> {
    let v = version_str.trim();

    // Reject v-prefixed version strings
    if v.starts_with('v') && v[1..].starts_with(|c: char| c.is_ascii_digit()) {
        anyhow::bail!("No Node.js release found matching '{v}'");
    }

    if v == "beta" {
        anyhow::bail!("'beta' channel is not supported for Node.js");
    }

    match v {
        "" | "latest" | "canary" | "next" | "nightly" | "edge" => {
            return releases
                .first()
                .map(|r| r.version.trim_start_matches('v').to_string())
                .ok_or_else(|| anyhow::anyhow!("No Node.js releases found"));
        }
        "lts" | "stable" | "current" => {
            return releases
                .iter()
                .find(|r| r.is_lts())
                .map(|r| r.version.trim_start_matches('v').to_string())
                .ok_or_else(|| anyhow::anyhow!("No LTS Node.js release found"));
        }
        _ => {}
    }

    // Strip trailing .x / .X suffix from partial versions
    let prefix = v.trim_end_matches(".x").trim_end_matches(".X");

    // Try to find the first release whose semver starts with `v{prefix}.`
    let needle = format!("v{prefix}.");
    releases
        .iter()
        .find(|r| r.version.starts_with(&needle) || r.version == format!("v{prefix}"))
        .map(|r| r.version.trim_start_matches('v').to_string())
        .ok_or_else(|| anyhow::anyhow!("No Node.js release found matching '{version_str}'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_release(version: &str, lts: serde_json::Value) -> NodeRelease {
        NodeRelease {
            version: version.to_string(),
            lts,
        }
    }

    fn stable_releases() -> Vec<NodeRelease> {
        vec![
            make_release("v22.0.0", json!(false)),
            make_release("v20.11.0", json!("Iron")),
            make_release("v20.10.0", json!("Iron")),
            make_release("v18.19.0", json!("Hydrogen")),
        ]
    }

    // --- is_lts / lts_name ---

    #[test]
    fn lts_name_returns_codename() {
        let r = make_release("v20.11.0", json!("Iron"));
        assert_eq!(r.lts_name(), Some("Iron"));
    }

    #[test]
    fn lts_name_returns_none_when_false() {
        let r = make_release("v22.0.0", json!(false));
        assert!(r.lts_name().is_none());
    }

    #[test]
    fn is_lts_true_for_lts_release() {
        let r = make_release("v20.11.0", json!("Iron"));
        assert!(r.is_lts());
    }

    #[test]
    fn is_lts_false_for_current_release() {
        let r = make_release("v22.0.0", json!(false));
        assert!(!r.is_lts());
    }

    // --- resolve_from ---

    #[test]
    fn resolve_latest_returns_first() {
        let r = resolve_from("latest", &stable_releases()).unwrap();
        assert_eq!(r, "22.0.0");
    }

    #[test]
    fn resolve_current_returns_lts() {
        // current now maps to the latest LTS, same as lts/stable
        let r = resolve_from("current", &stable_releases()).unwrap();
        assert_eq!(r, "20.11.0");
    }

    #[test]
    fn resolve_beta_returns_error() {
        assert!(resolve_from("beta", &stable_releases()).is_err());
    }

    #[test]
    fn resolve_lts_returns_first_lts() {
        let r = resolve_from("lts", &stable_releases()).unwrap();
        assert_eq!(r, "20.11.0");
    }

    #[test]
    fn resolve_stable_returns_first_lts() {
        let r = resolve_from("stable", &stable_releases()).unwrap();
        assert_eq!(r, "20.11.0");
    }

    #[test]
    fn resolve_major_returns_latest_in_major() {
        let r = resolve_from("20", &stable_releases()).unwrap();
        assert_eq!(r, "20.11.0");
    }

    #[test]
    fn resolve_major_x_notation() {
        let r = resolve_from("20.x", &stable_releases()).unwrap();
        assert_eq!(r, "20.11.0");
    }

    #[test]
    fn resolve_minor_prefix() {
        let r = resolve_from("20.10", &stable_releases()).unwrap();
        assert_eq!(r, "20.10.0");
    }

    #[test]
    fn resolve_exact_with_v_prefix_rejected() {
        // v-prefixed inputs are actively rejected
        assert!(resolve_from("v20.11.0", &stable_releases()).is_err());
    }

    #[test]
    fn resolve_unknown_errors() {
        assert!(resolve_from("99", &stable_releases()).is_err());
    }

    #[test]
    fn resolve_empty_list_errors() {
        assert!(resolve_from("lts", &[]).is_err());
    }

    #[test]
    fn resolve_canary_returns_first() {
        let r = resolve_from("canary", &stable_releases()).unwrap();
        assert_eq!(r, "22.0.0");
    }

    #[test]
    fn resolve_nightly_returns_first() {
        let r = resolve_from("nightly", &stable_releases()).unwrap();
        assert_eq!(r, "22.0.0");
    }

    #[test]
    fn resolve_edge_returns_first() {
        let r = resolve_from("edge", &stable_releases()).unwrap();
        assert_eq!(r, "22.0.0");
    }
}
