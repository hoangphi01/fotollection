// SPDX-License-Identifier: GPL-3.0-only
//! "Check for updates" (FR-061, ADR-0025). Only when the user clicks the button: one HTTPS
//! request to GitHub's latest-release API, then, if a newer version exists, its installer is
//! opened in the browser and the user runs it (the installers replace the old version, keeping
//! settings and projects). Nothing is downloaded or installed by the app, nothing runs on its
//! own, and the core never touches the network (NFR-006).

// Tauri commands take `State` by value.
#![allow(clippy::needless_pass_by_value)]

use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::state::{AppState, IpcError};

const LATEST: &str = "https://api.github.com/repos/hoangphi01/fotollection/releases/latest";
/// Every link the app may open starts here: the project's own releases.
const RELEASES: &str = "https://github.com/hoangphi01/fotollection/releases/";
/// Where "Report a problem" leads: the project's issue forms, bug or idea (FR-069).
const ISSUES: &str = "https://github.com/hoangphi01/fotollection/issues/new/choose";
/// Where the app's mark leads: the project's home page (owner, 2026-11-06).
const WEBSITE: &str = "https://fotollection.org/";
/// GitHub's answer is a few kilobytes; anything far larger is refused.
const MAX_BYTES: u64 = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(10);

/// The result shown under the button.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    /// This app's version, `0.7.0`.
    pub current: String,
    /// The latest published version, `0.8.0`.
    pub latest: String,
    /// True when `latest` is newer than `current`.
    pub newer: bool,
    /// The release page, for "What's new".
    pub page_url: String,
    /// The oldest version still safe to use, published with the release (FR-103); `None`
    /// when the release says none.
    pub min_safe: Option<String>,
    /// True when this version is older than `min_safe`: the notice turns into a red warning.
    pub below_min_safe: bool,
    /// Why, in English and Vietnamese, plain text (never HTML); empty when not said.
    pub reason_en: String,
    pub reason_vi: String,
}

/// What a release says about older versions (`.github/update.json`, carried in the release
/// notes as `<!-- fotollection-update {…} -->`, FR-103): one request still answers both.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UpdateNote {
    min_safe: String,
    #[serde(default)]
    reason: Reason,
}

#[derive(Deserialize, Default)]
struct Reason {
    #[serde(default)]
    en: String,
    #[serde(default)]
    vi: String,
}

const NOTE_START: &str = "<!-- fotollection-update ";
/// The longest reason kept, in characters.
const MAX_REASON: usize = 400;

/// The note in a release's `body`, if it is there and reads as one; anything else is ignored.
fn update_note(body: &str) -> Option<UpdateNote> {
    let start = body.find(NOTE_START)? + NOTE_START.len();
    let end = body[start..].find("-->")? + start;
    serde_json::from_str(body[start..end].trim()).ok()
}

/// A reason as plain, short text: control characters dropped, at most [`MAX_REASON`].
fn plain(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .take(MAX_REASON)
        .collect::<String>()
        .trim()
        .to_owned()
}

/// The installer to open after a check found a newer version, never supplied by the webview.
/// The installer and the release page found by the last check: (download, page).
#[derive(Default)]
pub struct PendingDownload(Mutex<Option<(String, String)>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Mac,
    Windows,
    /// Linux run as an `AppImage` (ADR-0034), on an ARM64 machine or not. A `.deb` install is
    /// `Other`: APT or the release page updates it.
    LinuxAppImage {
        arm: bool,
    },
    Other,
}

impl Platform {
    fn this() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some() {
            Self::LinuxAppImage {
                arm: cfg!(target_arch = "aarch64"),
            }
        } else {
            Self::Other
        }
    }

    /// The release asset for this system: the recommended installer.
    fn asset_suffix(self) -> Option<&'static str> {
        match self {
            Self::Mac => Some("_universal.dmg"),
            Self::Windows => Some("_x64-setup.exe"),
            Self::LinuxAppImage { arm: false } => Some("_amd64.AppImage"),
            Self::LinuxAppImage { arm: true } => Some("_aarch64.AppImage"),
            Self::Other => None,
        }
    }
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<Asset>,
    /// The release notes, which may carry the minimum safe version (FR-103).
    #[serde(default)]
    body: String,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Reads GitHub's answer without the network: the check to show and the link to open.
fn evaluate(
    json: &[u8],
    current: &str,
    platform: Platform,
) -> Result<(UpdateCheck, String), String> {
    let release: Release = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    if release.draft || release.prerelease {
        return Err("the latest release is not a stable version".into());
    }
    let parse =
        |v: &str| semver::Version::parse(v.trim_start_matches('v')).map_err(|e| e.to_string());
    let (latest, mine) = (parse(&release.tag_name)?, parse(current)?);
    if !release.html_url.starts_with(RELEASES) {
        return Err("unexpected release page".into());
    }
    let download = platform
        .asset_suffix()
        .and_then(|suffix| release.assets.iter().find(|a| a.name.ends_with(suffix)))
        .map(|a| a.browser_download_url.clone())
        .filter(|url| url.starts_with(RELEASES))
        .unwrap_or_else(|| release.html_url.clone());
    // A minimum above the latest release is a mistake in the note: ignored, never trusted.
    let note = update_note(&release.body).and_then(|n| {
        let min = parse(&n.min_safe).ok()?;
        (min <= latest).then_some((min, n.reason))
    });
    let (min_safe, below, reason) = match note {
        Some((min, reason)) => (Some(min.to_string()), mine < min, reason),
        None => (None, false, Reason::default()),
    };
    Ok((
        UpdateCheck {
            current: mine.to_string(),
            latest: latest.to_string(),
            newer: latest > mine && latest.pre.is_empty(),
            page_url: release.html_url,
            min_safe,
            below_min_safe: below,
            reason_en: plain(&reason.en),
            reason_vi: plain(&reason.vi),
        },
        download,
    ))
}

/// This build's version. A debug build can pretend to be older (`FOTOLLECTION_FAKE_VERSION`),
/// to try the "newer version" path and the red warning (FR-103).
fn current_version() -> String {
    #[cfg(debug_assertions)]
    if let Ok(fake) = std::env::var("FOTOLLECTION_FAKE_VERSION") {
        return fake;
    }
    env!("CARGO_PKG_VERSION").to_owned()
}

/// GitHub's latest release, or, in a debug build only, a file named by
/// `FOTOLLECTION_UPDATE_FEED` (FR-103), so the notice and the red warning can be tried with
/// no newer release out (`src-tauri/tests/fixtures/update/`). A release build never reads it.
fn fetch() -> Result<Vec<u8>, IpcError> {
    #[cfg(debug_assertions)]
    if let Some(feed) = std::env::var_os("FOTOLLECTION_UPDATE_FEED") {
        return std::fs::read(feed).map_err(|e| IpcError::new("update.offline", e.to_string()));
    }
    fetch_github()
}

fn fetch_github() -> Result<Vec<u8>, IpcError> {
    let offline = |e: ureq::Error| IpcError::new("update.offline", e.to_string());
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .user_agent(concat!(
                "Fotollection/",
                env!("CARGO_PKG_VERSION"),
                " (update check)"
            ))
            .build(),
    );
    let mut response = agent
        .get(LATEST)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(offline)?;
    response
        .body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_vec()
        .map_err(offline)
}

/// One check: GitHub's latest release read, and its installer and page kept for the buttons.
fn check_now(app: &AppHandle) -> Result<UpdateCheck, IpcError> {
    let json = fetch()?;
    let (check, download) = evaluate(&json, &current_version(), Platform::this())
        .map_err(|why| IpcError::new("update.unreadable", why))?;
    let pending = tauri::Manager::state::<AppState>(app);
    if let Ok(mut slot) = pending.update_download.0.lock() {
        *slot = check.newer.then(|| (download, check.page_url.clone()));
    }
    Ok(check)
}

/// Asks GitHub for the latest version, once, because the user clicked "Check for updates".
#[tauri::command]
#[specta::specta]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateCheck, IpcError> {
    tauri::async_runtime::spawn_blocking(move || check_now(&app))
        .await
        .map_err(|e| IpcError::internal(e.to_string()))?
}

/// The check at start (FR-103), once, when `ui.checkUpdatesAtStart` is on (the default): a
/// newer version, or `None` when there is none, the check is off, or the network or GitHub
/// does not answer, so an offline start shows nothing. It only reads; nothing is downloaded.
#[tauri::command]
#[specta::specta]
pub async fn check_for_update_at_start(app: AppHandle) -> Result<Option<UpdateCheck>, IpcError> {
    let on = tauri::Manager::state::<AppState>(&app)
        .ui
        .read()
        .is_ok_and(|ui| ui.check_updates_at_start);
    if !on {
        return Ok(None);
    }
    tauri::async_runtime::spawn_blocking(move || check_now(&app).ok().filter(|c| c.newer))
        .await
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// Opens the release page found by the last check (*What's new*), never a page the webview
/// names.
#[tauri::command]
#[specta::specta]
pub fn open_update_page(app: AppHandle, state: State<'_, AppState>) -> Result<(), IpcError> {
    let url = state
        .update_download
        .0
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|(_, page)| page.clone()))
        .ok_or_else(|| IpcError::new("update.none", "no newer version was found"))?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// Opens the installer found by the last check in the browser.
#[tauri::command]
#[specta::specta]
pub fn open_update_download(app: AppHandle, state: State<'_, AppState>) -> Result<(), IpcError> {
    let url = state
        .update_download
        .0
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|(download, _)| download.clone()))
        .ok_or_else(|| IpcError::new("update.none", "no newer version was found"))?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// Opens the project's issue page in the browser (FR-069). The address is fixed here, so the
/// webview cannot open any other; the app itself sends nothing.
#[tauri::command]
#[specta::specta]
pub fn open_issue_page(app: AppHandle) -> Result<(), IpcError> {
    app.opener()
        .open_url(ISSUES, None::<&str>)
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// Opens the project's home page in the browser, from a click on the app's mark (FR-099). The
/// address is fixed here, as `open_issue_page`'s; the app itself sends nothing.
#[tauri::command]
#[specta::specta]
pub fn open_website(app: AppHandle) -> Result<(), IpcError> {
    app.opener()
        .open_url(WEBSITE, None::<&str>)
        .map_err(|e| IpcError::internal(e.to_string()))
}

/// What *Copy diagnostics* copies: the version, the system and the last lines of the error
/// log, already free of paths (`error_log`). The user pastes it into an issue; the app sends
/// nothing.
#[tauri::command]
#[specta::specta]
pub fn diagnostics() -> String {
    fotollection_core::error_log::diagnostics(20)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, prerelease: bool) -> Vec<u8> {
        format!(
            r#"{{"tag_name":"{tag}","html_url":"{RELEASES}tag/{tag}","prerelease":{prerelease},
               "assets":[
                 {{"name":"Fotollection_{v}_universal.dmg","browser_download_url":"{RELEASES}download/{tag}/Fotollection_{v}_universal.dmg"}},
                 {{"name":"Fotollection_{v}_x64-setup.exe","browser_download_url":"{RELEASES}download/{tag}/Fotollection_{v}_x64-setup.exe"}},
                 {{"name":"Fotollection_{v}_x64_en-US.msi","browser_download_url":"{RELEASES}download/{tag}/Fotollection_{v}_x64_en-US.msi"}},
                 {{"name":"Fotollection_{v}_amd64.AppImage","browser_download_url":"{RELEASES}download/{tag}/Fotollection_{v}_amd64.AppImage"}},
                 {{"name":"Fotollection_{v}_aarch64.AppImage","browser_download_url":"{RELEASES}download/{tag}/Fotollection_{v}_aarch64.AppImage"}},
                 {{"name":"Fotollection_{v}_amd64.deb","browser_download_url":"{RELEASES}download/{tag}/Fotollection_{v}_amd64.deb"}}]}}"#,
            v = tag.trim_start_matches('v'),
        )
        .into_bytes()
    }

    /// FR-103: the minimum safe version travels in the release notes; below it, a warning;
    /// a missing, broken or impossible note is ignored.
    #[test]
    fn fr_103_the_minimum_safe_version_from_the_release_notes() -> Result<(), String> {
        let with_body = |body: &str| {
            let mut json: serde_json::Value =
                serde_json::from_slice(&release("v0.13.0", false)).map_err(|e| e.to_string())?;
            json["body"] = serde_json::Value::String(body.to_owned());
            serde_json::to_vec(&json).map_err(|e| e.to_string())
        };
        let note = r#"Highlights…

<!-- fotollection-update {"minSafe":"0.10.0","reason":{"en":"A fault that could stop the app.","vi":"Một lỗi có thể làm app dừng."}} -->"#;
        let (check, _) = evaluate(&with_body(note)?, "0.9.2", Platform::Mac)?;
        assert!(check.newer && check.below_min_safe);
        assert_eq!(check.min_safe.as_deref(), Some("0.10.0"));
        assert_eq!(check.reason_vi, "Một lỗi có thể làm app dừng.");
        // At or above the minimum: a quiet note only.
        let (check, _) = evaluate(&with_body(note)?, "0.10.0", Platform::Mac)?;
        assert!(check.newer && !check.below_min_safe);
        // No note, a broken one, or a minimum above the latest release: no warning.
        for body in [
            "no note",
            "<!-- fotollection-update {not json} -->",
            r#"<!-- fotollection-update {"minSafe":"9.0.0"} -->"#,
        ] {
            let (check, _) = evaluate(&with_body(body)?, "0.9.2", Platform::Mac)?;
            assert!(!check.below_min_safe && check.min_safe.is_none(), "{body}");
        }
        // The test feeds the dev app can read parse as releases.
        for feed in [
            include_str!("../tests/fixtures/update/newer.json"),
            include_str!("../tests/fixtures/update/below-min-safe.json"),
        ] {
            let (check, _) = evaluate(feed.as_bytes(), "0.12.1", Platform::Mac)?;
            assert!(check.newer);
        }
        Ok(())
    }

    #[test]
    fn fr_061_newer_versions_and_the_right_installer() -> Result<(), String> {
        let (check, download) = evaluate(&release("v0.8.0", false), "0.7.0", Platform::Mac)?;
        assert!(check.newer);
        assert_eq!(
            (check.current.as_str(), check.latest.as_str()),
            ("0.7.0", "0.8.0")
        );
        assert!(download.ends_with("Fotollection_0.8.0_universal.dmg"));
        let (_, download) = evaluate(&release("v0.8.0", false), "0.7.0", Platform::Windows)?;
        assert!(download.ends_with("_x64-setup.exe"));
        // An AppImage gets the AppImage for its architecture (ADR-0034).
        let intel = Platform::LinuxAppImage { arm: false };
        let (_, download) = evaluate(&release("v0.8.2", false), "0.8.1", intel)?;
        assert!(download.ends_with("Fotollection_0.8.2_amd64.AppImage"));
        let arm = Platform::LinuxAppImage { arm: true };
        let (_, download) = evaluate(&release("v0.8.2", false), "0.8.1", arm)?;
        assert!(download.ends_with("Fotollection_0.8.2_aarch64.AppImage"));
        // No installer for this system (a .deb install): the release page.
        let (check, download) = evaluate(&release("v0.8.0", false), "0.7.0", Platform::Other)?;
        assert_eq!(download, check.page_url);
        Ok(())
    }

    #[test]
    fn fr_061_same_or_older_is_up_to_date() -> Result<(), String> {
        assert!(
            !evaluate(&release("v0.7.0", false), "0.7.0", Platform::Mac)?
                .0
                .newer
        );
        assert!(
            !evaluate(&release("v0.6.0", false), "0.7.0", Platform::Mac)?
                .0
                .newer
        );
        // A beta of this app is older than the release it leads to.
        assert!(
            evaluate(&release("v0.7.0", false), "0.7.0-beta.1", Platform::Mac)?
                .0
                .newer
        );
        Ok(())
    }

    /// The real request, run by hand only (`cargo test -p fotollection -- --ignored live`):
    /// tests never go online by themselves.
    #[test]
    #[ignore = "goes online"]
    fn fr_061_live_latest_release() -> Result<(), String> {
        let json = fetch().map_err(|e| e.message)?;
        let (check, download) = evaluate(&json, "0.6.0", Platform::Mac)?;
        eprintln!("{check:?} → {download}");
        assert!(check.newer);
        assert!(download.starts_with(RELEASES) && download.ends_with("_universal.dmg"));
        Ok(())
    }

    #[test]
    fn fr_061_untrusted_answers_are_refused() {
        // Pre-releases, links outside the project, broken or odd JSON: errors, never a panic.
        assert!(evaluate(&release("v0.8.0-beta.1", true), "0.7.0", Platform::Mac).is_err());
        let foreign = br#"{"tag_name":"v9.0.0","html_url":"https://example.com/x","assets":[]}"#;
        assert!(evaluate(foreign, "0.7.0", Platform::Mac).is_err());
        let bad_asset = format!(
            r#"{{"tag_name":"v0.8.0","html_url":"{RELEASES}tag/v0.8.0","assets":[{{"name":"a_universal.dmg","browser_download_url":"https://example.com/a.dmg"}}]}}"#
        );
        let (check, download) = evaluate(bad_asset.as_bytes(), "0.7.0", Platform::Mac)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            download, check.page_url,
            "a foreign asset link falls back to the page"
        );
        assert!(evaluate(b"{not json", "0.7.0", Platform::Mac).is_err());
        assert!(
            evaluate(
                br#"{"tag_name":"latest","html_url":"x"}"#,
                "0.7.0",
                Platform::Mac
            )
            .is_err()
        );
    }
}
