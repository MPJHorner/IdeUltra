//! Update checker: ask the GitHub Releases API once on startup whether a
//! newer tag is available, surface a banner if so. Pure version-compare
//! lives in this file and is unit-tested; the I/O wrapper shells out to
//! `curl` (built into macOS) so we don't carry an HTTPS client in our
//! dependency tree.
//!
//! This is intentionally NOT "auto-update": clicking the banner opens the
//! GitHub release page in the browser. Real Sparkle-style replace-in-place
//! is a follow-up that needs notarised binaries to be useful.

use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

pub const REPO: &str = "MPJHorner/IdeUltra";

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub html_url: String,
    /// Best-effort first .dmg asset, if the release attached one. Surface
    /// for a future "download in place" upgrade flow.
    #[allow(dead_code)]
    pub dmg_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<GhReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GhReleaseAsset {
    name: String,
    browser_download_url: String,
}

/// Fetch the latest release for `repo` (e.g. `"MPJHorner/IdeUltra"`) and
/// compare against `current_version` (e.g. `env!("CARGO_PKG_VERSION")`).
/// Returns `Ok(None)` when no newer version is available, `Ok(Some)` when
/// one is. Network failures or rate-limit errors propagate as `Err`.
pub fn check(repo: &str, current_version: &str) -> Result<Option<UpdateInfo>> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let output = Command::new("curl")
        .args([
            "-sSL",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: IdeUltra-update-check",
            &url,
        ])
        .output()
        .context("spawn curl")?;
    if !output.status.success() {
        anyhow::bail!(
            "curl failed (exit {:?})",
            output.status.code()
        );
    }
    let release: GhRelease = serde_json::from_slice(&output.stdout)
        .with_context(|| "parse GitHub release JSON")?;

    if !is_newer(&release.tag_name, current_version) {
        return Ok(None);
    }
    let dmg_url = release
        .assets
        .iter()
        .find(|a| a.name.to_lowercase().ends_with(".dmg"))
        .map(|a| a.browser_download_url.clone());
    Ok(Some(UpdateInfo {
        current: current_version.to_string(),
        latest: release.tag_name,
        html_url: release.html_url,
        dmg_url,
    }))
}

/// Open `url` in the user's default browser. Best-effort; logs on failure.
pub fn open_in_browser(url: &str) {
    if let Err(err) = Command::new("open").arg(url).status() {
        tracing::warn!(error = %err, url = %url, "open url failed");
    }
}

/// True if `latest` represents a strictly newer version than `current`.
/// Both inputs may carry a leading `v`. Non-numeric trailing components
/// (e.g. `0.18.0-rc.1`) are ignored after the first non-numeric token.
pub fn is_newer(latest: &str, current: &str) -> bool {
    parts(latest) > parts(current)
}

fn parts(v: &str) -> Vec<u32> {
    let v = v.trim_start_matches('v').trim_start_matches('V');
    let mut out = Vec::with_capacity(3);
    for piece in v.split('.') {
        match piece.parse::<u32>() {
            Ok(n) => out.push(n),
            Err(_) => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn major_bump_is_newer() {
        assert!(is_newer("1.0.0", "0.99.99"));
    }

    #[test]
    fn minor_bump_is_newer() {
        assert!(is_newer("0.19.0", "0.18.0"));
    }

    #[test]
    fn patch_bump_is_newer() {
        assert!(is_newer("0.18.1", "0.18.0"));
    }

    #[test]
    fn same_version_not_newer() {
        assert!(!is_newer("0.18.0", "0.18.0"));
    }

    #[test]
    fn older_not_newer() {
        assert!(!is_newer("0.17.9", "0.18.0"));
    }

    #[test]
    fn v_prefix_is_stripped() {
        assert!(is_newer("v0.19.0", "0.18.0"));
        assert!(is_newer("v0.19.0", "v0.18.0"));
    }

    #[test]
    fn prerelease_suffix_is_ignored() {
        // Both parse as 0.19.0 once the trailing -rc.1 / -beta is dropped.
        assert!(!is_newer("0.19.0-rc.1", "0.19.0"));
        assert!(is_newer("0.19.0-rc.1", "0.18.0"));
    }

    #[test]
    fn extra_minor_components_treated_correctly() {
        // 1.2.3.4 should compare as > 1.2.3.
        assert!(is_newer("1.2.3.4", "1.2.3"));
        assert!(!is_newer("1.2.3", "1.2.3.4"));
    }

    #[test]
    fn unparseable_versions_compare_as_empty_vecs() {
        // Both unparseable: equal, so not newer.
        assert!(!is_newer("garbage", "garbage"));
        // Real version vs garbage: real is newer.
        assert!(is_newer("0.1.0", "garbage"));
    }
}
