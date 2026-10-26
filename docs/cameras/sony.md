# Sony — protocol notes

> **Status:** Observing · **Last reviewed:** 2026-10-26
>
> What Sony bodies declare and do over USB, written in our own words from observation only ([ADR-0026](../adr/0026-cameras-through-the-os-usb-stack.md), clean-room rule in [README.md §4](README.md#4-clean-room-rule)). Raw dumps and USB captures stay on the owner's machine and are never committed.

## ILCE-6400 (a6400), firmware 2.0, USB Connection: PC Remote

Observed with `probe_camera` on 2026-10-26 (macOS, ImageCaptureCore).

- **Device info:** manufacturer "Sony Corporation", PTP 1.00, vendor extension ID `0x00000011`, version 100, description "Sony PTP Extensions", functional mode 0.
- **Standard operations (11):** GetDeviceInfo, OpenSession, CloseSession, GetStorageIDs, GetStorageInfo, GetNumObjects, GetObjectHandles, GetObjectInfo, GetObject, GetThumb, GetPartialObject. No `InitiateCapture`, no property operations.
- **Maker operations (5):** `0x9201`, `0x9202`, `0x9205`, `0x9207`, `0x9209`.
- **Events:** maker events `0xC201`, `0xC202`, `0xC203` only.
- **Device properties:** none declared in the standard list (they are expected behind the maker's operations).
- **Image formats:** `0x3801` (EXIF/JPEG), `0xB301`, `0xB101` (maker-defined; expected to be RAW and its preview, to confirm).
- **Behaviour:** after the session opens the body shows "Connecting… USB" and keeps the shutter locked; pressing it does nothing and no event arrives in 30 s. It waits for a handshake through the maker's operations.

## Next

USB capture on Windows (Wireshark + USBPcap) of Imaging Edge Remote connecting, showing live view, taking a JPEG and a RAW, and changing ISO once. From it: the handshake, the capture sequence, how live-view frames are fetched, and which events announce a new photo. Each finding goes here, then into `crates/oscamera/src/vendor/sony.rs`.
