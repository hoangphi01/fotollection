# Panasonic — protocol notes

> **Status:** Observing · **Last reviewed:** 2026-10-26
>
> What Panasonic bodies declare and do over USB, written in our own words from observation only ([ADR-0026](../adr/0026-cameras-through-the-os-usb-stack.md), clean-room rule in [README.md §4](README.md#4-clean-room-rule)).

## DC-GX9, firmware 1.5

The body offers two USB modes: **PC (Storage)** (USB mass storage, no PTP) and **PictBridge (PTP)**. There is no tether mode.

### PictBridge (PTP)

Observed with `probe_camera` on 2026-10-26 (macOS, ImageCaptureCore).

- **Device info:** manufacturer "Panasonic", PTP 1.00, vendor extension ID `0x00000006` (the MTP one) with an empty description, functional mode 0.
- **Standard operations (16):** GetDeviceInfo, OpenSession, CloseSession, GetStorageIDs, GetStorageInfo, GetNumObjects, GetObjectHandles, GetObjectInfo, GetObject, GetThumb, SendObjectInfo, SendObject, GetDevicePropDesc, GetDevicePropValue, SetDevicePropValue, GetPartialObject.
- **Maker operations:** none. No `InitiateCapture`.
- **Events:** StoreAdded, StoreRemoved, DeviceInfoChanged, RequestObjectTransfer. No ObjectAdded.
- **Device properties:** `0xD406`, `0xD407` (PictBridge printing).
- **Image formats:** `0x3001` (association), `0x3002` (script), `0x3006` (DPOF), `0x3801` (EXIF/JPEG), `0x380D` (TIFF).
- **Behaviour:** the screen shows only the PictBridge logo and the shutter is locked. Right after the session opens, ImageCaptureCore lists the card's files and the body answers no command until that is done (10 s timeout hit with 70 files; 20 ms once the list was complete).

### Powered on with the cable plugged in

Turning the body off, plugging the cable in, then turning it on starts it in shooting mode. In that state it shows **no USB device at all**: neither ImageCaptureCore nor the system's USB registry (`ioreg -p IOUSB`) lists anything, so the cable carries power only (checked 2026-10-26).

### Conclusion

Over USB the GX9 can only hand over files that are already on the card, and only in a mode where the shutter is locked: no remote capture, no live view, no new-photo event; while it can shoot, it is not on the USB bus. The only way to make it shoot from the computer and get the photo at once is its Wi-Fi remote protocol (the one Panasonic's Image App uses), which would need an exception to the wired-first rule and to NFR-006 (a direct link to the camera, never the internet). An HDMI capture card gives live view, but that is what any streaming tool already does, so it is not a Fotollection feature.
