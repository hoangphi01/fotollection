// SPDX-License-Identifier: GPL-3.0-only
//! Renders one photo exactly as export would, optionally with the info frame (FR-040).
//!
//! `cargo run --release -p fotollection-core --example render_one -- <in.jpg> <out.jpg> [crop|fit|none] [10x15] [degrees]`
//!
//! `degrees` straightens the photo (FR-006); the render time is printed for the benchmark log.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use fotollection_core::PrintSize;
use fotollection_core::codec::default_codec;
use fotollection_core::frame::{CameraProfile, FrameFit, FrameSettings, build_text};
use fotollection_core::image::apply_orientation;
use fotollection_core::import::check_file;
use fotollection_core::library::Library;
use fotollection_core::metadata;
use fotollection_core::plan::{FrameRequest, PlanInput, plan};
use fotollection_core::render::render;
use fotollection_core::settings::RenderSettings;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (input, output) = (&args[1], &args[2]);
    let fit = match args.get(3).map(String::as_str) {
        Some("fit") => Some(FrameFit::Fit),
        Some("crop") => Some(FrameFit::Crop),
        _ => None,
    };
    let print = PrintSize::by_id(args.get(4).map_or("10x15", String::as_str)).expect("print size");
    let straighten: f32 = args.get(5).map_or(0.0, |d| d.parse().expect("degrees"));

    let file = check_file(input.as_ref()).expect("valid JPEG");
    let meta = metadata::read_path(&file.path);
    let mut lib = Library::new();
    let folder = lib.add_folder(file.path.parent().expect("parent"));
    let id = lib.push(folder, &file, &meta).expect("photo");
    let record = lib.record(id).expect("record");

    let codec = default_codec();
    let bytes = std::fs::read(input).unwrap();
    let t = std::time::Instant::now();
    let decoded = fotollection_core::codec::decode_image(codec.as_ref(), &bytes).unwrap();
    eprintln!(
        "decode {}x{}: {:.0} ms",
        decoded.image.width,
        decoded.image.height,
        t.elapsed().as_secs_f64() * 1e3
    );
    let upright = apply_orientation(decoded.image, record.orientation);

    let mut settings = RenderSettings::default();
    settings.stamp.enabled = fit.is_none();
    let request = fit.map(|fit| {
        let frame = FrameSettings {
            enabled: true,
            fit,
            ..FrameSettings::default()
        };
        // Demo profile: a manual lens and a photo style that EXIF can't know.
        let mut profile =
            CameraProfile::new(1, record.shot.make.clone(), record.shot.model.clone());
        profile.lens = Some("Индустар-50-2".into());
        profile.focal_mm = Some(50.0);
        profile.f_number = Some(4.0);
        profile.photo_style = Some("L. Monochrome D".into());
        FrameRequest {
            text: build_text(
                &record.shot,
                Some(&profile),
                &frame,
                Some(record.capture),
                None,
            ),
            // The bundled brand logo, as the app uses by default (ADR-0020).
            logo: profile
                .brand()
                .and_then(|b| fotollection_core::brands::logo(b, false, frame.logo_style).ok()),
            settings: frame,
            font: None,
        }
    });
    let p = plan(&PlanInput {
        width: upright.width,
        height: upright.height,
        capture: Some(record.capture),
        settings: &settings,
        print,
        max_long_edge: None,
        frame: request.as_ref(),
        crop: None,
        place: None,
        straighten,
    });
    let t = std::time::Instant::now();
    let image = render(&upright, &p).unwrap();
    eprintln!(
        "render {straighten}°: {:.0} ms",
        t.elapsed().as_secs_f64() * 1e3
    );
    std::fs::write(
        output,
        codec
            .encode(
                &image,
                fotollection_core::codec::EncodeOptions::print(92),
                None,
            )
            .unwrap(),
    )
    .unwrap();
    println!(
        "{}x{} → {}x{} (ratio {:.4}, print {}:{})",
        upright.width,
        upright.height,
        image.width,
        image.height,
        f64::from(image.width.max(image.height)) / f64::from(image.width.min(image.height)),
        print.long_mm,
        print.short_mm
    );
}
