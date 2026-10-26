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

### Imaging Edge Remote over USB, first capture (2026-10-26, Windows 10, USBPcap)

The first recording ran 94 s. In it Remote connected, showed live view, and one RAW photo was taken **with the body's own shutter button**. Nothing was shot from Remote and no setting was changed, so neither appears below. The device is `054c:0caa`. Bulk OUT `0x02` carries commands, bulk IN `0x81` carries data and responses, and interrupt IN `0x83` carries events.

- **Session:** Windows opens the session (`OpenSession` 1, `GetDeviceInfo`, `GetStorageIDs`) as soon as the body is plugged in. Remote sends no `OpenSession` of its own. Its first transaction continues Windows's transaction IDs, which suggests it goes through the system's WPD stack rather than a driver of its own (to confirm).
- **Handshake** (about 1 s, all answered `OK`):
  1. `0x9201` (1, 0, 0) → 8 bytes of zeros
  2. `0x9201` (2, 0, 0) → 8 bytes of zeros
  3. `0x9202` (`0xC8`) → 136 bytes: a list of property codes, starting with the standard `0x5004`, `0x5005`, `0x5007`, `0x500A`, `0x500B`, `0x500C`, `0x500E`, `0x5010`, `0x5013`, then maker codes `0xD2xx`
  4. `0x9201` (3, 0, 0) → 8 bytes of zeros
  5. `0x9202` (`0xC8`) again → same list

  The body starts sending events once this is done.
- **Settings:** `0x9209` (no parameters) returns every property's current state in one block of about 1.7 KB. Remote polls it about every 0.2 s, alternating with live-view frames. Its layout is still to be written down.
- **Live view:** `GetObjectInfo` and then `GetObject` on the fixed handle `0xFFFFC002`. The info gives format `0x3801`. Each data block starts with two 32-bit values: an offset (1012) and a length. Offset plus length equals the block size, so the frame is the last *length* bytes (expected to be a JPEG; to confirm with a frame dump). Frames were 30–45 KB, up to 125 KB once. Remote fetched about 4.8 frames/s while also polling `0x9209`.
- **Events:** `0xC203` (property changed, parameter = property code, such as `0x5004` or `0xD21D`) arrives several times a second. On a shot from the body:
  1. `0xC201` (`0xFFFFC001`) announces the new photo.
  2. About 0.1 s later Remote calls `GetObjectInfo` and `GetObject` on handle `0xFFFFC001`. The info gives format `0xB101` and 24,959,744 bytes, so `0xB101` is the RAW (ARW) file. The name begins with "DSC0".
  3. `0xC202` (`0xFFFFC001`) follows about 1 s after `0xC201`.

  While the photo was being made, one live-view `GetObject` was answered `0x200F` (Access Denied).

### Imaging Edge Remote over USB, second capture (2026-10-26)

The second recording ran 48 s and started with Remote already connected, so it shows no handshake. Remote was still open when the recording stopped. Everything was done in Remote: four presses of its shutter button and four steps of shutter speed.

- **Controls:** `0x9207` (control code) sends a 2-byte value in its data phase and returns `OK`. It is a separate operation from reading the settings.
- **Shutter button** (`0xD2C1` and `0xD2C2`, each press about 0.25 s):
  1. `0xD2C1` = 2
  2. `0xD2C2` = 2
  3. `0xD2C2` = 1
  4. `0xD2C1` = 1

  This reads as half press down, full press down, full press up, then half press up (to confirm).
- **Results of the four presses:** the first two (10.8 s and 16.7 s) produced no `0xC201` and no photo. The last two produced one photo each:
  - at 21.2 s: a smaller file, read in about 0.65 s, expected to be the JPEG;
  - at 36.2 s: format `0xB101`, 24,824,576 bytes, the RAW.

  For both, `0xC201` came about 0.8 s after the press and `0xC202` about 0.8 s after that. A press that does not shoot (no focus, still busy) has to be detected from the missing `0xC201`.
- **Shutter speed:** `0xD20D` = `0xFFFF` (−1), sent once per step in Remote, four times. It is a relative step, not an absolute value; `+1` is expected to step the other way. Which property it changes is to be read from the `0x9209` block (to confirm).

### Imaging Edge Remote over USB, third capture (2026-10-26)

The third recording ran 55 s, again starting with Remote already connected. Everything was done in Remote, in this order:

1. ISO down two steps.
2. A shot in RAW.
3. The format changed to JPEG, then a shot.
4. The format changed back to RAW, then a shot.

All three presses gave a photo.

- **ISO:** `0x9207` with control code `0xD21E`, once per step. The data phase is 4 bytes; the first two are `0xFFFF` (−1) and the last two differed between the two sends (`0x0000`, `0x0011`). Read it as a 16-bit step plus padding (to confirm).
- **Setting an absolute value:** `0x9205` (property code, here `0x5004`) sends a 2-byte value in its data phase. On this body, `0x5004` = `0x02` gave JPEG and `0x10` gave RAW. Remote sent each value twice, about 0.15 s apart. So `0x9205` sets a value and `0x9207` acts like a button or a dial step.
- **What the files start with:** the RAW (`0xB101`, 24,734,464 bytes) starts with a little-endian TIFF header (`49 49 2A 00`), as ARW does. The JPEG (`0x3801`, 4,073,521 bytes) starts with `FF D8 FF E1` (JPEG with EXIF).
- **Timings, measured from the first shutter command:**

  | Format | Photo announced (`0xC201`) | `GetObject` | Photo fully read |
  |---|---:|---:|---:|
  | RAW | 0.82–0.84 s | 0.69 s (about 36 MB/s) | 1.7 s |
  | JPEG | 1.07 s | 0.14 s (about 29 MB/s) | 1.36 s |

- **Recording limit:** USBPcap kept only the start of each large transfer (about 6 MB of the 24.7 MB RAW). The timings come from the commands and responses and are complete. The file contents are not in the recording, and nothing here depends on them.

### The settings block (`0x9209`), decoded from the recordings

The block holds a 32-bit count (62 on the a6400) and 4 bytes we do not use. Then come the entries, one per count. Each entry is a standard PTP property description (ISO 15740 §5.5.3) with one extra byte after get/set:

1. property code (2 bytes)
2. data type (2 bytes)
3. get/set (1 byte)
4. Sony's extra byte (1 byte)
5. default value
6. current value
7. form: 0 = none; 1 = range (min, max, step); 2 = list (a 16-bit count, then the values)

Decoding the a6400's block this way uses exactly all of its 1,730 bytes. Comparing successive blocks while Remote changed things identifies:

| Code | Type | Get/set | What | Values seen |
|---|---|---|---|---|
| `0x5004` | u8 | 1 | File format | List of 7; `0x02` JPEG, `0x10` RAW |
| `0x5007` | u16 | 0 | Aperture, f-number × 100 (standard code) | 200 = f/2.0 |
| `0xD20D` | u32 | 0 | Shutter speed: numerator in the high 16 bits, denominator in the low 16 | 1/250 → 1/160 → 1/100 over four steps of −1 |
| `0xD21E` | u32 | 0 | ISO | 1600 → 1250 → 1000 over two steps of −1; a list of 89 values, `0xFFFFFF` among them (expected to mean Auto) |
| `0xD215` | u16 | 0 | Changes on every shot: 0 → 1 → `0x8001` → 0, around `0xC201` | Meaning to confirm |
| `0xD2C1`, `0xD2C2` | u16 | `0x81` | Shutter half and full press | 1 up, 2 down |

Get/set is 1 for values set with `0x9205` and `0x81` for buttons used with `0x9207`. ISO and shutter speed show 0, yet they move by steps with `0x9207`. Several other codes also show `0x81` (`0xD2C3`, `0xD2C5`, `0xD2C7`–`0xD2D4`, `0xD2D9`) and are left unnamed until observed.

### In code

`crates/oscamera/src/vendor/sony.rs` encodes these notes:

- the handshake sequence;
- the shutter press;
- the `0x9207` and `0x9205` data phases;
- the settings block parser (bounded; tested on a hand-made block, and checked once against a recorded a6400 block that is not committed);
- the live-view frame's offset and length;
- shutter-speed decoding;
- the gap between shutter steps and the shot state (from the first run below).

`crates/oscamera/src/windows.rs` carries these operations through WPD. `examples/sony_remote.rs` drives the body with them.

### First run from our own code (2026-10-26, Windows 10, WPD)

`sony_remote` on the a6400 (firmware 2.0, PC Remote) through Windows's own driver, with Imaging Edge closed. Measured on a Lenovo 20U9002NUS: i7-10610U, 4 cores, 16 GB, Windows 10 IoT Enterprise LTSC.

- **Session and handshake:** WPD opens the device in 1–2 ms. The handshake takes 20–60 ms, or 0.96 s the first time after plugging in, all steps answered `OK`. No `OpenSession` is sent; WPD's session is used.
- **Settings:**
  - The block read right after the first handshake had empty values (ISO 0, shutter `0xFFFF/0xFFFF`, f/0). On later runs it was filled at once. Read it again before trusting it.
  - Setting the file format with `0x9205` shows in the block at once, but the **next shot still used the old format**. The shot after it used the new one. Remote sent the value twice and shot seconds later; our code must wait for the format to take effect (exact delay to confirm).
- **Shutter:**
  - The four `0x9207` steps sent back to back (about 40 ms) shot nothing.
  - With 300 ms or 500 ms between steps, the body focused and shot every time (4 of 4).
  - `SHUTTER_STEP_GAP` is 300 ms.
- **Noticing the shot:**
  - Through WPD, `0xC201` never arrived, in four tries.
  - `0xD215` in the polled settings block went to `0x8001` 6–30 ms after the last shutter step, and the photo could then be read. That is the signal used.
  - WPD does deliver `0xC203` (property changed), as GUID `C2030000-5738-4FF2-8445-BE3126691059`: the event code is in the high 16 bits of the GUID's first field. These events stopped arriving while the settings block was polled every 150 ms.
- **Photos read:**

  | Format | `GetObject` | Rate | From the first shutter step to the photo in memory |
  |---|---:|---:|---:|
  | JPEG, 4.3 MB | 145–148 ms | 29 MB/s | 1.12–1.72 s |
  | RAW (ARW), 24.8 MB | 759–763 ms | 32.6 MB/s | 1.72–1.75 s |

  The 0.9–1.5 s before the read is the press itself with its gaps.
  - Sizes matched `GetObjectInfo`.
  - The JPEG starts `FF D8 FF E1` and opens as 6000 × 4000.
  - The ARW starts with a TIFF header.
  - `GetObjectInfo` gives the file name (`DSC04099.JPG`) but a width and height of 0.
- **Live view:**
  - 100 frames in 3.3 s = **30.3 fps**, 1024 × 680 JPEG, 78 KB mean.
  - The slowest successful request took 29 ms.
  - A request made before the next frame is ready is answered `0x200F` (715 times in that run) and costs almost nothing. Polling is the way to pace it.
  - The time from scene to screen was not measured; it needs a clock in the scene.

## Next

- Find why `0xC201` does not come through WPD (another event GUID, or a property of the event registration), or keep the shot-state poll.
- The delay before a new file format applies.
- Record a disconnect: Remote closed before Ctrl+C.
- Name `0xC202` and the other `0x81` buttons when they are observed.
- Note for other Sony bodies: Imaging Edge installs a libusbK driver (`srcameradriver.inf`, "Sony Remote Control Camera") for 65 other models, not the a6400. Most of their USB product IDs are above the a6400's `0x0CAA` (for example `0x0CCC`, `0x0D18`, `0x0E08`), which suggests newer bodies. On a PC where that driver has taken such a body, WPD does not see it. Newer bodies may need a different path in PC Remote mode. The a6400's path may still work for bodies of its generation (a6100, a6600, a7 III, RX100 VI/VII), but this is not checked yet.

### Other Sony bodies: a read-only remote probe (planned)

The question above can be answered without having the body here:

1. **The probe.** A `--probe` mode for `sony_remote` that never shoots and never sets anything. It prints:
   - the body's USB product ID and the driver Windows bound to it;
   - whether WPD sees it;
   - if WPD sees it, the handshake result and the settings block.
2. **The download.** It ships as one prebuilt `.exe`, with three steps for whoever runs it.
3. **The run.** That person puts the body in USB Connection: PC Remote, runs the file and sends back the text it prints. This is observation, within the clean-room rule.
4. **If WPD does not see the body,** a USB recording of Imaging Edge Remote with that body follows ([capture-guide.md](capture-guide.md)).

First candidate: a ZV-E10 (2021), lent by a friend of the owner.
