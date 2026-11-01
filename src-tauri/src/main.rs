// SPDX-License-Identifier: GPL-3.0-only
// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    nvidia_webkit_workaround();
    fotollection_lib::run();
}

/// `WebKitGTK`'s DMA-BUF renderer leaves the window blank with Nvidia's driver (ADR-0034).
/// Where that driver is loaded and the user has not chosen, the app starts itself again
/// with the renderer off: setting the variable in this process would need `unsafe` in Rust
/// 2024, and `exec` replaces the process, so nothing else runs twice. Other machines keep
/// the faster renderer.
#[cfg(target_os = "linux")]
fn nvidia_webkit_workaround() {
    use std::os::unix::process::CommandExt;
    const VAR: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
    if std::env::var_os(VAR).is_some()
        || !std::path::Path::new("/proc/driver/nvidia/version").exists()
    {
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let error = std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .env(VAR, "1")
        .exec();
    // `exec` returns only when it failed: carry on with the default renderer.
    eprintln!("could not restart with {VAR}=1: {error}");
}
