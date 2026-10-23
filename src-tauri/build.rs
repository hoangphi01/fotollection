// SPDX-License-Identifier: GPL-3.0-only
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut attributes = tauri_build::Attributes::new();

    // tauri-build only embeds the Windows app manifest into the app binary, so test
    // executables that link Tauri fail to load (STATUS_ENTRYPOINT_NOT_FOUND). Embed our
    // own manifest into every binary instead, as documented for
    // `WindowsAttributes::new_without_app_manifest`.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        let manifest = std::env::current_dir()?.join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }

    tauri_build::try_build(attributes)?;
    Ok(())
}
