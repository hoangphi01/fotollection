# Cameras over USB — Fotollection

> **Status:** Planned · **Last reviewed:** 2026-10-26
>
> What each camera can do when plugged into Fotollection by cable, and how we know. The plan and its reasons are in [ADR-0026](../adr/0026-cameras-through-the-os-usb-stack.md); the requirements are [FR-062](../PRD.md#fr-062--live-view-from-a-camera), [FR-063](../PRD.md#fr-063--tethered-capture-over-usb) and [FR-064](../PRD.md#fr-064--watch-a-folder-for-new-photos). Each maker's protocol, once observed, gets its own page here (`sony.md`, `panasonic.md`, …), written in our own words.

## 1. Paths, in order of preference

All of them are wired. Wi-Fi is a last resort only (latency).

| Path | Photo for print | Live view | Needs |
|---|---|---|---|
| **A. Our PTP driver** over the OS USB stack (ImageCaptureCore, WPD) | Full resolution, at once | Yes, with the maker's live-view operation | A driver module for the maker (`crates/oscamera/src/vendor/`) |
| B. Standard webcam (UVC) through `getUserMedia` | No (about 2 MP video) | Yes | A body with UVC |
| C. HDMI capture card (UVC) | No | Yes, any body with clean HDMI out | A 10–20 € USB capture card |
| D. Watched folder + the maker's wired tether app | Full resolution | No | The maker's app installed |

## 2. Support table

Filled in by Step 0 and later tests. "—" = not checked yet.

| Body | USB modes | Declared vendor ID / ops | Capture from the app | Camera's shutter reported | Live view (fps, size, latency) | Shot to file (JPEG / RAW) | Path |
|---|---|---|---|---|---|---|---|
| Sony a6400 (ILCE-6400) | PC Remote, MTP, Mass Storage | — | — | — | — | — | A expected; B via Imaging Edge Webcam (1024 × 576) |
| Panasonic GX9 (DC-GX9) | PC, PictBridge | — | — | — | — | — | A if it declares capture and live-view ops; otherwise C for live view and D or plain import for photos |

Known from makers' documents (not yet tested here): UVC on Canon R50/R8/R6 II, Nikon Z50II/Z5II, Fujifilm X-T5/X-S20/X100VI, Panasonic S1 II, OM-5 ([list](https://diiorio.me/posts/uvc-cameras/)); LUMIX Webcam Software and LUMIX Tether do **not** list the GX9.

## 3. Step 0: what to measure

`crates/oscamera/examples/probe_camera.rs` (not written yet) prints the results; the owner holds the cameras, on macOS first, then Windows. Each body, each USB mode, one camera at a time:

1. **Declared operations:** the full `GetDeviceInfo` (vendor extension ID, operations, events, properties). This tells whether the GX9 has capture and live view over USB at all.
2. **Capture from the computer:** standard `InitiateCapture` first, then the maker's operations (Sony's half/full press; Panasonic's 0x94xx range if declared); time from the command to the full file on disk, JPEG and RAW.
3. **The camera's own shutter:** is a "new object" event sent? If not, capture from the computer is the path.
4. **Live view:** frame rate, size and latency through the maker's operation.
5. **USB capture on Windows** (Wireshark + USBPcap): Imaging Edge Remote driving the a6400 (capture, live view, ISO and aperture changes), and any Panasonic app that drives the GX9. Captures stay local and are never committed; only our protocol notes and small hand-made response samples (test fixtures) are.
6. **HDMI capture card**, if one is at hand: is the output clean (no overlays), and what is the latency?

Results go to [RESULT.md](../RESULT.md) §13, a row in [OPTIMIZATION.md §5](../OPTIMIZATION.md#5-benchmark-log), and the table above.

### Before a session

- **Disk:** at least 10 GB free (`scripts/resources.sh`). On 2026-10-26 swap held 21 GB of the disk; a restart frees it.
- **a6400:** Menu → Setup → **USB Connection: PC Remote**; PC Remote Function → **Still Img. Save Dest.: PC+Camera**; a charged battery; a data cable, not a charge-only one.
- **GX9:** Setup → **USB Mode**: try PC, then PictBridge.

## 4. Clean-room rule

Protocols are learned by observing the camera and the maker's own app, written down here in our words, and coded from these notes. No code from libgphoto2 or any maker SDK is read into the project, so the code stays GPL-3.0-only and relicensable ([ADR-0008](../adr/0008-license-gpl3-with-relicensing-option.md)).
