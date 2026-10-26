// SPDX-License-Identifier: GPL-3.0-only
//! Drives a Sony body in "PC Remote" USB mode with Fotollection's own driver
//! (`vendor::sony`, docs/cameras/sony.md): handshake, settings, a shot saved to disk, live view.
//! Prints what happened and how long it took, for NFR-018 and NFR-019. Nothing is written
//! unless `--out` is given. Close Imaging Edge Remote first: one app drives the camera at a time.
//!
//! ```sh
//! cargo run -p fotollection-oscamera --example sony_remote -- [--format jpeg|raw] [--shoot]
//!     [--live N] [--out DIR] [--wait 10] [--hold ms]
//! ```
//!
//! A shot is noticed by polling the shot state, as Remote polls the settings; `--event-only`
//! waits for the `NEW_PHOTO` event instead, `--no-poll` stops polling while waiting, and
//! `--events` prints every event (diagnostics for the transport).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::similar_names
)]

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn main() {
    use std::time::{Duration, Instant};

    #[cfg(target_os = "macos")]
    use fotollection_oscamera::macos::{Browser, Camera, Event, pump};
    use fotollection_oscamera::ptp::{self, ObjectInfo, op};
    use fotollection_oscamera::vendor::sony::{
        self, HANDSHAKE, SHUTTER_PRESS, SHUTTER_STEP_GAP, Settings, control_data, ev, file_format,
        handle, live_view_image, object_format, prop, set_value_data, shutter_speed,
    };
    #[cfg(target_os = "windows")]
    use fotollection_oscamera::windows::{Browser, Camera, Event, pump};

    const T: Duration = Duration::from_secs(10);

    fn settings(camera: &mut Camera) -> Vec<sony::Property> {
        let reply = camera
            .send(sony::op::ALL_SETTINGS, &[], None, T)
            .expect("0x9209");
        Settings::parse(&reply.data).expect("settings block")
    }

    fn current(props: &[sony::Property], code: u16) -> Option<i128> {
        props
            .iter()
            .find(|p| p.code == code)
            .and_then(|p| p.current.as_int())
    }

    fn print_settings(props: &[sony::Property]) {
        let format = match current(props, prop::FILE_FORMAT).and_then(|v| u16::try_from(v).ok()) {
            Some(file_format::JPEG) => "JPEG".to_string(),
            Some(file_format::RAW) => "RAW".to_string(),
            other => format!("{other:?}"),
        };
        let iso = current(props, prop::ISO).map_or("?".into(), |v| {
            if v == 0x00FF_FFFF {
                "Auto".into()
            } else {
                v.to_string()
            }
        });
        let speed = current(props, prop::SHUTTER_SPEED)
            .and_then(|v| u32::try_from(v).ok())
            .and_then(shutter_speed)
            .map_or("?".into(), |(n, d)| {
                if n == 1 {
                    format!("1/{d}")
                } else {
                    format!("{n}/{d}")
                }
            });
        let f = current(props, prop::F_NUMBER)
            .map_or("?".into(), |v| format!("f/{}.{}", v / 100, (v % 100) / 10));
        println!(
            "Settings: {} properties; format {format}, ISO {iso}, shutter {speed}, {f}",
            props.len()
        );
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let wait = Duration::from_secs(value("--wait").and_then(|v| v.parse().ok()).unwrap_or(10));
    let out = value("--out").map(std::path::PathBuf::from);

    let browser = Browser::start().expect("camera browser");
    println!("Looking for cameras (10 s)…");
    let mut cameras = browser.cameras(Duration::from_secs(10));
    let index = cameras
        .iter()
        .position(|c| (c.name() + &c.product_kind()).contains("Sony"))
        .unwrap_or(0);
    if cameras.is_empty() {
        println!(
            "No camera found. Check USB Connection: PC Remote, the cable, and that Remote is closed."
        );
        return;
    }
    let mut camera = cameras.swap_remove(index);
    println!(
        "Camera: {} ({}) {:?}",
        camera.name(),
        camera.product_kind(),
        camera.capabilities()
    );

    let t = Instant::now();
    camera
        .open(Duration::from_secs(15))
        .expect("open the camera");
    println!("Session open in {} ms", t.elapsed().as_millis());

    let t = Instant::now();
    for (code, params) in HANDSHAKE {
        let step = Instant::now();
        let reply = camera.send(code, params, None, T).expect("handshake step");
        println!(
            "  0x{code:04X} {params:X?}: response 0x{:04X}, {} bytes, {} ms",
            reply.response.code,
            reply.data.len(),
            step.elapsed().as_millis()
        );
    }
    println!("Handshake in {} ms", t.elapsed().as_millis());

    let t = Instant::now();
    let props = settings(&mut camera);
    println!("Settings block read in {} ms", t.elapsed().as_millis());
    print_settings(&props);

    if let Some(format) = value("--format") {
        let want = if format.eq_ignore_ascii_case("raw") {
            file_format::RAW
        } else {
            file_format::JPEG
        };
        let reply = camera
            .send(
                sony::op::SET_VALUE,
                &[u32::from(prop::FILE_FORMAT)],
                Some(&set_value_data(want)),
                T,
            )
            .expect("set the file format");
        println!("Set format: response 0x{:04X}", reply.response.code);
        let end = Instant::now() + Duration::from_secs(3);
        while Instant::now() < end
            && current(&settings(&mut camera), prop::FILE_FORMAT) != Some(i128::from(want))
        {
            pump(Duration::from_millis(100));
        }
        print_settings(&settings(&mut camera));
    }

    if flag("--shoot") {
        pump(Duration::from_millis(200));
        let _ = camera.take_events();
        // `--hold ms` overrides the gap between press steps, which gives autofocus time.
        let hold = value("--hold")
            .and_then(|v| v.parse().ok())
            .map_or(SHUTTER_STEP_GAP, Duration::from_millis);
        let press = Instant::now();
        for (i, (code, value)) in SHUTTER_PRESS.into_iter().enumerate() {
            if i > 0 {
                pump(hold);
            }
            let reply = camera
                .send(
                    sony::op::CONTROL,
                    &[u32::from(code)],
                    Some(&control_data(code, value)),
                    T,
                )
                .expect("shutter");
            if !reply.response.is_ok() {
                println!(
                    "  shutter 0x{code:04X}={value}: response 0x{:04X}",
                    reply.response.code
                );
            }
        }
        println!("Shutter pressed in {} ms", press.elapsed().as_millis());

        // Remote reads the settings block while it waits; do the same.
        let mut announced = None;
        let end = press + wait;
        let mut shot_state = None;
        'wait: while Instant::now() < end {
            if !flag("--no-poll")
                && let Ok(reply) = camera.send(sony::op::ALL_SETTINGS, &[], None, T)
                && let Ok(props) = Settings::parse(&reply.data)
            {
                let state = current(&props, prop::SHOT_STATE);
                if state != shot_state {
                    println!(
                        "  +{} ms  shot state 0x{:04X}",
                        press.elapsed().as_millis(),
                        state.unwrap_or(0)
                    );
                    shot_state = state;
                }
                if !flag("--event-only") && state.is_some_and(sony::photo_ready) {
                    announced = Some(press.elapsed());
                    break 'wait;
                }
            }
            for timed in camera.take_events() {
                let ms = timed.at.duration_since(press).as_millis();
                match &timed.event {
                    Event::Ptp(raw) => {
                        let Ok(e) = ptp::event(raw) else { continue };
                        if flag("--events") || e.code != ev::PROPERTY_CHANGED {
                            println!("  +{ms} ms  event 0x{:04X} {:X?}", e.code, e.params);
                        }
                        if e.code == ev::NEW_PHOTO {
                            announced = Some(timed.at.duration_since(press));
                            break 'wait;
                        }
                    }
                    Event::Removed => {
                        println!("Camera removed");
                        return;
                    }
                    #[allow(unreachable_patterns)]
                    _ => {}
                }
            }
            pump(Duration::from_millis(150));
        }
        let Some(announced) = announced else {
            println!(
                "No new photo within {wait:?}: the camera did not shoot (focus not found, or busy)."
            );
            return;
        };
        println!("New photo announced after {} ms", announced.as_millis());

        let reply = camera
            .send(op::GET_OBJECT_INFO, &[handle::PHOTO], None, T)
            .expect("GetObjectInfo");
        let info = ObjectInfo::parse(&reply.data).expect("object info");
        let kind = match info.format {
            object_format::JPEG => "JPEG",
            object_format::RAW => "RAW",
            _ => "other",
        };
        println!(
            "Photo: {} ({kind}, 0x{:04X}), {} bytes, {}×{}",
            info.filename, info.format, info.size, info.width, info.height
        );
        let read = Instant::now();
        let reply = camera
            .send(
                op::GET_OBJECT,
                &[handle::PHOTO],
                None,
                Duration::from_secs(60),
            )
            .expect("GetObject");
        let read_ms = read.elapsed().as_millis();
        let bytes = reply.data.len();
        println!(
            "Read {bytes} bytes in {read_ms} ms ({:.1} MB/s); {} the declared size; head {:02X?}",
            bytes as f64 / 1e6 / (read_ms.max(1) as f64 / 1000.0),
            if bytes == info.size as usize {
                "matches"
            } else {
                "does NOT match"
            },
            &reply.data[..reply.data.len().min(4)]
        );
        if let Some(dir) = &out {
            let name = if info.filename.is_empty() {
                format!("shot.{}", if kind == "RAW" { "ARW" } else { "JPG" })
            } else {
                info.filename.clone()
            };
            let path = dir.join(name);
            std::fs::write(&path, &reply.data).expect("write the photo");
            println!("Saved {}", path.display());
        }
        println!(
            "Shutter to photo in memory: {} ms",
            press.elapsed().as_millis()
        );
    }

    if let Some(n) = value("--live").and_then(|v| v.parse::<u32>().ok()) {
        let start = Instant::now();
        let mut sizes = Vec::new();
        let mut slowest = Duration::ZERO;
        let mut last = Vec::new();
        let mut refused = std::collections::BTreeMap::<String, u32>::new();
        // Until `n` good frames or 30 s: a request faster than the camera makes frames fails.
        while sizes.len() < n as usize && start.elapsed() < Duration::from_secs(30) {
            let t = Instant::now();
            let _ = camera.send(op::GET_OBJECT_INFO, &[handle::LIVE_VIEW], None, T);
            match camera.send(op::GET_OBJECT, &[handle::LIVE_VIEW], None, T) {
                Ok(reply) if reply.response.is_ok() => {
                    slowest = slowest.max(t.elapsed());
                    if let Ok(image) = live_view_image(&reply.data) {
                        sizes.push(image.len());
                        last = image.to_vec();
                    }
                }
                Ok(reply) => {
                    *refused
                        .entry(format!("response 0x{:04X}", reply.response.code))
                        .or_default() += 1;
                }
                Err(e) => *refused.entry(e.to_string()).or_default() += 1,
            }
        }
        let secs = start.elapsed().as_secs_f64();
        let frames = sizes.len();
        let mean = sizes.iter().sum::<usize>() / frames.max(1);
        println!(
            "Live view: {frames} frames in {secs:.2} s = {:.1} fps; mean {} KB, slowest good request {} ms; JPEG start {}; refused {refused:?}",
            frames as f64 / secs,
            mean / 1024,
            slowest.as_millis(),
            last.starts_with(&[0xFF, 0xD8])
        );
        if let Some(dir) = &out {
            let path = dir.join("live-view.jpg");
            std::fs::write(&path, &last).expect("write the frame");
            println!("Saved {}", path.display());
        }
    }
    println!("Done.");
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn main() {
    println!("sony_remote runs on macOS and Windows.");
}
