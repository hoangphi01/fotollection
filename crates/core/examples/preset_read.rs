// SPDX-License-Identifier: GPL-3.0-only
//! What a Lightroom preset file holds as Fotollection reads it (FR-089): its name, group,
//! process and settings, or why it is refused. Files are only read.
//!
//! `cargo run -p fotollection-core --example preset_read -- <file.xmp|file.lrtemplate|file.zip>…`

fn show(
    name: &str,
    read: Result<fotollection_core::preset::Preset, fotollection_core::preset::PresetError>,
) {
    match read {
        Ok(p) => {
            println!(
                "{name}: {:?} · group {:?} · process {:?} · {} settings",
                p.name,
                p.group,
                p.process,
                p.values.len()
            );
            for (k, v) in &p.values {
                println!("  {k} = {v:?}");
            }
        }
        Err(e) => println!("{name}: refused ({e})"),
    }
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(bytes) = std::fs::read(&path) else {
            println!("{path}: unreadable");
            continue;
        };
        if path.to_ascii_lowercase().ends_with(".zip") {
            match fotollection_core::preset::parse_zip(&bytes) {
                Ok(all) => all.into_iter().for_each(|(n, r)| show(&n, r)),
                Err(e) => println!("{path}: refused ({e})"),
            }
        } else {
            show(&path, fotollection_core::preset::parse(&bytes));
        }
    }
}
