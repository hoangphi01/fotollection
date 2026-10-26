# ADR-0026: Cameras over USB through the OS imaging stack, with our own vendor drivers

- **Status:** Proposed (Step 0 measurements on a Panasonic GX9 and a Sony a6400 decide the details)
- **Date:** 2026-10-26
- **Related:** FR-062, FR-063, FR-064, NFR-018, NFR-019, ADR-0008, ADR-0017, ADR-0023, [cameras/README.md](../cameras/README.md)

## Context

The owner wants a camera plugged in **by cable** to work like a webcam in Fotollection: a live view to frame the shot, a **Capture** button in the app (or the camera's own shutter, where it reports the shot), and the full-resolution photo appearing at once to crop and print, as in a photo booth at a print shop or an event. It must work with customers' cameras of every brand, not only the owner's. Wi-Fi is too slow and is a last resort only. Reverse-engineering the makers' protocols is acceptable.

No single driver makes every camera a webcam. Research on 2026-10-26:

- **UVC (standard webcam over USB):** recent bodies only (Canon R50/R8/R6 II, Nikon Z50II/Z5II, Fujifilm X-T5/X-S20/X100VI, Panasonic S1 II, OM-5), video around 1080p (about 2 MP): enough to frame, not to print. Neither the GX9 nor the a6400 has it ([diiorio.me list](https://diiorio.me/posts/uvc-cameras/)).
- **Vendor webcam apps** (Sony Imaging Edge Webcam, Canon EOS Webcam Utility, LUMIX Webcam Software): a virtual webcam, the user installs it; the a6400 is supported at 1024 × 576, the GX9 is not.
- **PTP** (the protocol cameras speak over USB): the standard operations cover capture and "new object" events; live view and remote shutter on many bodies need the maker's own operations. macOS exposes PTP through ImageCaptureCore (`ICCameraDevice`, `requestSendPTPCommand`); Windows through WPD, whose MTP extension commands carry any operation, vendor ones included. Neither needs a driver to be installed.
- **libgphoto2** supports many bodies but is LGPL, which [ADR-0008](0008-license-gpl3-with-relicensing-option.md) and [ADR-0017](0017-heic-through-os-codecs.md) keep out, and has no official Windows build. **Maker SDKs** (Canon EDSDK, Sony Camera Remote SDK, Nikon, Fujifilm) are proprietary and cannot ship in a GPL program.
- **Replacing the USB driver** (WinUSB/libusb through Zadig) would make users install a driver and break Windows Photos and Explorer for that camera.

## Options considered

1. **Our own PTP drivers on the OS USB stack** (ImageCaptureCore, WPD), standard operations plus each maker's operations, documented by us from observation. Full-resolution photos, remote capture, live view; no install, no LGPL, no proprietary code. Costs reverse-engineering per maker and a camera to test each one.
2. **UVC and HDMI capture cards through `getUserMedia`.** Live view for any body with UVC or a clean HDMI output (both the GX9 and the a6400 have micro-HDMI); no stills for print.
3. **A watched folder** fed by the maker's own wired tether app (Imaging Edge Remote, EOS Utility, LUMIX Tether). Full-resolution photos for any body its maker supports; no live view in Fotollection; the user installs the maker's app.
4. **libgphoto2 or maker SDKs.** Rejected: licence (LGPL, proprietary) and no Windows build for libgphoto2.
5. **WinUSB/libusb with a replaced driver.** Rejected: an install step and broken system apps.
6. **Wi-Fi transfer.** Rejected as a main path: latency; recorded only as a last resort.

## Decision

Option 1 is the main path, with options 2 and 3 as fallbacks, all wired. A new crate `crates/oscamera` holds three layers: `transport` (ImageCaptureCore on macOS, WPD on Windows; the only `unsafe`), `ptp` (pure Rust packing and parsing, tested on recorded responses) and `vendor` (one module per maker behind one trait, plus a `generic` one for standard PTP). Sony and Panasonic come first because their bodies are at hand; Canon and Nikon follow once a body can be borrowed.

**Clean-room rule.** Each maker's protocol is learned by observation only: what the camera declares (`GetDeviceInfo`), and USB captures of the maker's own app driving it (Wireshark + USBPcap on Windows). It is written down in our own words in `docs/cameras/<maker>.md`, and the code is written from those notes. No code from libgphoto2 or any SDK is read into the project. Raw captures are never committed. Observation for interoperability is what the EU Software Directive (2009/24/EC, Art. 5(3) and 6) allows a lawful user; this is a note, not legal advice.

## Consequences

- Step 0 (`crates/oscamera/examples/probe_camera.rs`) measures before any feature is built: declared operations, capture from the computer, the camera's own shutter, live view frame rate and latency, shot-to-file time for JPEG and RAW ([cameras/README.md](../cameras/README.md), [OPTIMIZATION.md §5.1](../OPTIMIZATION.md#51-planned-measurements)). Its results set NFR-018 and NFR-019 and decide the GX9's path.
- Camera data is untrusted input: every length and count from a PTP response is bounds-checked, packet sizes are capped, and bad data is an error, never a panic ([SECURITY.md](../SECURITY.md)).
- Live view and camera permission (macOS `NSCameraUsageDescription` for the webcam path) are local only: frames are shown, never stored or sent ([NFR-006](../PRD.md#nfr-006--offline-and-zero-telemetry)).
- Each supported body needs a real camera to test; the support table in `docs/cameras/README.md` says what each body can do and how it was checked.
- When this is accepted, ARCHITECTURE gains the crate and its IPC commands, and the status above becomes Accepted.
