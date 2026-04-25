/// Returns the Node.js platform string for the current OS.
/// e.g. "linux", "darwin", "win"
pub const fn os_str() -> &'static str {
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(target_os = "macos")]
    return "darwin";
    #[cfg(target_os = "windows")]
    return "win";
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return "linux"; // fallback
}

/// Returns the Node.js architecture string.
/// e.g. "x64", "arm64", "armv7l"
pub const fn arch_str() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    return "x64";
    #[cfg(target_arch = "aarch64")]
    return "arm64";
    #[cfg(target_arch = "arm")]
    return "armv7l";
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "arm",
    )))]
    return "x64"; // fallback
}

/// Returns the archive extension for the current platform.
/// Windows uses ".zip", everything else uses ".tar.gz".
pub const fn archive_ext() -> &'static str {
    #[cfg(target_os = "windows")]
    return "zip";
    #[cfg(not(target_os = "windows"))]
    return "tar.gz";
}

/// Builds the full download URL for a specific Node.js release.
///
/// # Example
/// ```
/// let url = n::arch::download_url("v20.11.0");
/// ```
#[must_use]
pub fn download_url(version_tag: &str) -> String {
    let os = os_str();
    let arch = arch_str();
    let ext = archive_ext();
    let filename = format!("node-{version_tag}-{os}-{arch}.{ext}");
    format!("https://nodejs.org/dist/{version_tag}/{filename}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_str_is_non_empty() {
        assert!(!os_str().is_empty());
    }

    #[test]
    fn os_str_is_known_value() {
        assert!(["linux", "darwin", "win"].contains(&os_str()));
    }

    #[test]
    fn arch_str_is_non_empty() {
        assert!(!arch_str().is_empty());
    }

    #[test]
    fn arch_str_is_known_value() {
        assert!(["x64", "arm64", "armv7l"].contains(&arch_str()));
    }

    #[test]
    fn archive_ext_is_non_empty() {
        assert!(!archive_ext().is_empty());
    }

    #[test]
    fn archive_ext_is_known_value() {
        assert!(["tar.gz", "zip"].contains(&archive_ext()));
    }

    #[test]
    fn download_url_contains_version() {
        let url = download_url("v20.11.0");
        assert!(url.contains("v20.11.0"), "url: {url}");
    }

    #[test]
    fn download_url_starts_with_nodejs_dist() {
        let url = download_url("v20.11.0");
        assert!(url.starts_with("https://nodejs.org/dist/"));
    }

    #[test]
    fn download_url_contains_os_and_arch() {
        let url = download_url("v20.11.0");
        assert!(url.contains(os_str()));
        assert!(url.contains(arch_str()));
    }

    #[test]
    fn download_url_ends_with_archive_ext() {
        let url = download_url("v20.11.0");
        assert!(url.ends_with(archive_ext()), "url: {url}");
    }
}
