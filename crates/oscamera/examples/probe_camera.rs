// SPDX-License-Identifier: GPL-3.0-only
//! Step 0 of the camera work (docs/cameras/README.md §3): what a camera plugged in by USB
//! declares, whether the computer can make it shoot, and how long a shot takes to be reported.
//! Prints only; nothing is written unless `--dump` is given. macOS for now.
//!
//! ```sh
//! cargo run -p fotollection-oscamera --example probe_camera -- [--dump info.bin]
//!     [--settle ms] [--capture | --icc-capture] [--watch 20]
//! ```
//!
//! `--capture` sends the standard `InitiateCapture`; `--icc-capture` uses `ImageCaptureCore`'s
//! own `requestTakePicture`. `--watch N` keeps listening N seconds for the camera's events
//! (press the shutter on the camera to see whether it reports the shot).

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(target_os = "macos")]
fn main() {
    use std::time::{Duration, Instant};

    use fotollection_oscamera::macos::{Browser, Event, pump};
    use fotollection_oscamera::ptp::{self, DeviceInfo, op};

    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let watch = value("--watch")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20u64);

    let browser = Browser::start().expect("main thread");
    println!("Looking for cameras (10 s)…");
    let mut cameras = browser.cameras(Duration::from_secs(10));
    let Some(camera) = cameras.first_mut() else {
        println!("No camera found. Check the USB mode on the camera and the cable.");
        return;
    };
    println!("Camera: {} ({})", camera.name(), camera.product_kind());
    println!("ImageCaptureCore capabilities: {:?}", camera.capabilities());

    let t = Instant::now();
    if let Err(e) = camera.open(Duration::from_secs(15)) {
        println!("Could not open a session: {e}");
        return;
    }
    println!("Session open in {} ms", t.elapsed().as_millis());
    if let Some(ms) = value("--settle").and_then(|v| v.parse().ok()) {
        println!("Waiting {ms} ms before the first command…");
        pump(Duration::from_millis(ms));
        for timed in camera.take_events() {
            println!("  while waiting: {:?}", timed.event);
        }
    }

    let t = Instant::now();
    let reply = match camera.send(op::GET_DEVICE_INFO, &[], None, Duration::from_secs(20)) {
        Ok(r) => r,
        Err(e) => {
            println!("GetDeviceInfo failed: {e}");
            return;
        }
    };
    println!(
        "GetDeviceInfo: response 0x{:04X}, {} bytes, {} ms",
        reply.response.code,
        reply.data.len(),
        t.elapsed().as_millis()
    );
    if let Some(path) = value("--dump") {
        std::fs::write(&path, &reply.data).expect("write the dump");
        println!("Raw DeviceInfo written to {path} (local only, never commit it)");
    }
    match DeviceInfo::parse(&reply.data) {
        Ok(info) => print_info(&info),
        Err(e) => println!("DeviceInfo could not be read: {e}"),
    }

    pump(Duration::from_millis(500));
    let _ = camera.take_events();

    let start = Instant::now();
    if flag("--capture") {
        match camera.send(op::INITIATE_CAPTURE, &[0, 0], None, Duration::from_secs(15)) {
            Ok(r) => println!(
                "InitiateCapture: response 0x{:04X} after {} ms",
                r.response.code,
                start.elapsed().as_millis()
            ),
            Err(e) => println!("InitiateCapture failed: {e}"),
        }
    } else if flag("--icc-capture") {
        camera.take_picture();
        println!("requestTakePicture sent");
    }

    println!("Listening for events for {watch} s (press the shutter on the camera)…");
    let end = Instant::now() + Duration::from_secs(watch);
    while Instant::now() < end {
        pump(Duration::from_millis(50));
        for timed in camera.take_events() {
            let ms = timed.at.duration_since(start).as_millis();
            match timed.event {
                Event::ItemsAdded(items) => println!("+{ms} ms  new items: {items:?}"),
                Event::Ptp(raw) => match ptp::event(&raw) {
                    Ok(e) => println!(
                        "+{ms} ms  PTP event 0x{:04X} {} {:X?}",
                        e.code,
                        ptp::event_name(e.code).unwrap_or("(maker's)"),
                        e.params
                    ),
                    Err(_) => println!("+{ms} ms  PTP event, {} raw bytes", raw.len()),
                },
                Event::Removed => {
                    println!("+{ms} ms  camera removed");
                    return;
                }
            }
        }
    }
    println!("Done.");
}

#[cfg(target_os = "macos")]
fn print_info(info: &fotollection_oscamera::ptp::DeviceInfo) {
    use fotollection_oscamera::ptp;
    println!();
    println!("Manufacturer: {}", info.manufacturer);
    println!("Model: {}", info.model);
    println!("Firmware: {}", info.device_version);
    println!(
        "PTP {}.{:02}, vendor extension 0x{:08X} ({}) v{}: {:?}",
        info.standard_version / 100,
        info.standard_version % 100,
        info.vendor_extension_id,
        ptp::vendor_name(info.vendor_extension_id),
        info.vendor_extension_version,
        info.vendor_extension_desc
    );
    println!("Functional mode: 0x{:04X}", info.functional_mode);
    let (standard, maker): (Vec<u16>, Vec<u16>) = info
        .operations
        .iter()
        .partition(|&&c| ptp::operation_name(c).is_some());
    println!("Standard operations ({}):", standard.len());
    for c in &standard {
        println!("  0x{c:04X} {}", ptp::operation_name(*c).unwrap_or(""));
    }
    println!("Maker operations ({}): {}", maker.len(), hex(&maker));
    println!("Events ({}): {}", info.events.len(), hex(&info.events));
    println!(
        "Properties ({}): {}",
        info.properties.len(),
        hex(&info.properties)
    );
    println!("Capture formats: {}", hex(&info.capture_formats));
    println!("Image formats: {}", hex(&info.image_formats));
    println!();
}

#[cfg(target_os = "macos")]
fn hex(codes: &[u16]) -> String {
    codes
        .iter()
        .map(|c| format!("{c:04X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("probe_camera runs on macOS for now; the Windows transport (WPD) comes next.");
}
