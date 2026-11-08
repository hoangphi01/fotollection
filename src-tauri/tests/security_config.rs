// SPDX-License-Identifier: GPL-3.0-only
//! The app's security configuration, checked on every `cargo test` (SECURITY.md §2, §7): the
//! content security policy, the window and webview settings, and the capabilities granted to
//! the webview. A change here is a security decision: it changes SECURITY.md in the same
//! commit, and this file says what is allowed.

use serde_json::Value;

fn read(path: &str) -> Value {
    let full = format!("{}/{path}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("{full}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{full}: {e}"))
}

fn csp() -> Vec<(String, Vec<String>)> {
    let conf = read("tauri.conf.json");
    let csp = conf["app"]["security"]["csp"]
        .as_str()
        .unwrap_or_else(|| panic!("a CSP is set"));
    csp.split(';')
        .filter_map(|d| {
            let mut words = d.split_whitespace();
            let name = words.next()?.to_owned();
            Some((name, words.map(str::to_owned).collect()))
        })
        .collect()
}

fn directive(name: &str) -> Vec<String> {
    csp()
        .into_iter()
        .find(|(n, _)| n == name)
        .map_or_else(|| panic!("CSP has {name}"), |(_, v)| v)
}

/// C2: the CSP allows the app's own page, its IPC and its photo protocol, nothing remote,
/// no script but its own files.
#[test]
fn security_the_csp_allows_only_the_app_itself() {
    assert_eq!(directive("default-src"), ["'self'"]);
    assert_eq!(
        directive("script-src"),
        ["'self'"],
        "no inline or eval scripts"
    );
    for locked in ["object-src", "base-uri", "form-action", "frame-ancestors"] {
        assert_eq!(directive(locked), ["'none'"], "{locked}");
    }
    let connect = directive("connect-src");
    for needed in [
        "ipc:",
        "http://ipc.localhost",
        "fotothumb:",
        "http://fotothumb.localhost",
    ] {
        assert!(
            connect.contains(&needed.to_owned()),
            "connect-src needs {needed}: the UI fetches it"
        );
    }
    for (name, sources) in csp() {
        for s in &sources {
            assert!(!s.contains("unsafe-eval"), "{name}: unsafe-eval");
            let remote = (s.starts_with("http://") || s.starts_with("https://"))
                && !s.ends_with(".localhost");
            assert!(
                !remote && s != "*" && s != "https:" && s != "http:",
                "{name}: a remote source {s}"
            );
        }
    }
}

/// C3: no global Tauri object, no developer tools, no asset protocol, no remote IPC, the
/// page from the built files, and frozen prototypes.
#[test]
fn security_the_app_config_opens_nothing() {
    let conf = read("tauri.conf.json");
    let app = &conf["app"];
    assert_ne!(app["withGlobalTauri"], Value::Bool(true), "withGlobalTauri");
    assert_ne!(
        app["security"]["assetProtocol"]["enable"],
        Value::Bool(true),
        "asset protocol"
    );
    assert!(
        app["security"]["dangerousRemoteDomainIpcAccess"].is_null(),
        "remote IPC"
    );
    for w in app["windows"].as_array().into_iter().flatten() {
        assert_ne!(w["devtools"], Value::Bool(true), "a window with devtools");
        assert!(
            w["url"].as_str().is_none_or(|u| !u.starts_with("http")),
            "a window on a remote URL"
        );
    }
    let dist = conf["build"]["frontendDist"].as_str().unwrap_or_default();
    assert!(
        !dist.starts_with("http"),
        "the page must come from the built files, not {dist}"
    );
}

/// C4: the webview's capabilities are exactly the ones SECURITY.md lists: events and the
/// title bar's window controls, for the main window, local only.
#[test]
fn security_the_webview_gets_only_its_listed_permissions() {
    const ALLOWED: &[&str] = &[
        "core:event:allow-listen",
        "core:event:allow-unlisten",
        "core:event:allow-emit",
        "core:window:allow-is-maximized",
        "core:window:allow-minimize",
        "core:window:allow-toggle-maximize",
        "core:window:allow-internal-toggle-maximize",
        "core:window:allow-close",
        "core:window:allow-start-dragging",
    ];
    let dir = format!("{}/capabilities", env!("CARGO_MANIFEST_DIR"));
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        seen += 1;
        let cap: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_default())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(
            cap["remote"].is_null(),
            "{}: remote URLs get IPC",
            path.display()
        );
        assert_eq!(
            cap["windows"],
            serde_json::json!(["main"]),
            "{}: windows",
            path.display()
        );
        for p in cap["permissions"].as_array().into_iter().flatten() {
            let name = p
                .as_str()
                .or_else(|| p["identifier"].as_str())
                .unwrap_or_default();
            assert!(
                ALLOWED.contains(&name),
                "{}: {name} is not in SECURITY.md's list",
                path.display()
            );
        }
    }
    assert!(seen > 0, "no capability file read");
}
