# Error control — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-02
>
> The dictionary of every error the app can show: the single source of truth for error codes. Fotollection is offline and sends nothing (NFR-006), so there is no Sentry. **The code a user reads out, `ERR-IMG-0002` for example, is the only clue we get.** It must say at once what went wrong, where it was raised, why, and what to do. Every new error gets its row here in the same commit as the code ([DOCS_GUIDE.md](DOCS_GUIDE.md) §2).

## 1. The code

`ERR-<KIND>-<NNNN>`: the kind of fault, then a four-digit number within that kind.

- **Permanent.** A number is never renumbered or reused, like FR, NFR and ADR IDs ([CLAUDE.md](../CLAUDE.md) rule 2). An error the app no longer raises keeps its row, marked *Retired in vX.Y.Z*.
- **One code, one meaning.** Two causes that need different fixes get two codes, even when the message is the same.
- **New codes go at the end of their kind.** The tables are listed by number.
- **Every code maps one stable Rust code** (`export.empty`, `camera.timeout`…), the one the interface already translates through `errors.<code>` in `ui/src/locales/*.json` ([SECURITY.md](SECURITY.md) §8). The Rust code stays for translation; the ERR code is what users and logs carry.

| Kind | What fails | Examples |
|------|------------|----------|
| **IMG** | The photo file itself | damaged or cut-off JPEG, a layout no decoder proves, a format not read, an image over the size cap |
| **RUS** | The Rust core | a caught panic, a poisoned lock, a broken invariant, a failed render or encode |
| **HDW** | Hardware | no camera answering, a card or drive removed during work, a camera still busy |
| **DRV** | Drivers | camera USB/PTP, WebKitGTK (Linux), WIC (Windows) |
| **EXT** | Extensions and outside codecs | Microsoft's HEIF/HEVC and Raw Image Extensions, the system libheif (FR-068) |
| **FS** | The file system | no permission, a read-only or full destination, a file that could not be written |
| **PRJ** | Project files (`.fotollection`) | not a project, damaged, saved by a newer version |
| **CFG** | Settings, fonts and logos | a setting out of range, a font or logo the app cannot use |
| **EXP** | Export | no destination, nothing to export, cancelled |
| **NET** | The network: *Check for updates* only | offline, an answer that cannot be read |
| **IPC** | Invalid input from the interface | a stale photo id, a selection over 10,000 photos |

## 2. What each row says

Code · since which version users see it · Rust code · where it is raised (`file:line`, the first place) · what the user sees (English; the Vietnamese is in `ui/src/locales/vi.json` under the same key) · the cause · what the user does. How to debug a code, what the log line carries and the fixture that reproduces it go in the row's cause, in [LESSONS.md](LESSONS.md) when a debugging session found it, and in the test that pins it.

## 3. Status of the plan (v0.9.1)

The codes below are assigned; the app shows them from **v0.9.1**:

1. **In the code:** `IpcError` and the core's error enums carry an `ErrCode`; the bindings give the interface the same constants. A test (or `scripts/check-docs.sh`) fails when a code in the code has no row here, or a row here has no code and is not retired.
2. **On screen:** every error message ends with its code, for example "The photo could not be decoded (ERR-IMG-0002)", and a click copies it.
3. **An offline log in place of Sentry:** a rotating error log (5 files of 1 MB) in the app's data folder. Each line has the time, the code, the app version, the system, and technical context such as the JPEG layout, the RAW sensor kind or the decoder used. **It never holds image data, GPS, or full paths** (only the file name's last part) ([SECURITY.md](SECURITY.md)).
4. **Copy diagnostics:** a button next to *Report a problem* in the versions popover copies the version, the system and the last codes, for the user to paste into an issue. The app sends nothing by itself.
5. **The panic hook** (SECURITY S7) writes an `ERR-RUS` line with a short backtrace instead of closing silently.

## 4. Dictionary

### IMG: The photo file itself

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-IMG-0001` | v0.9.1 | `culling.analysis_failed` | `crates/core/src/culling/mod.rs:50` | A photo could not be analysed | One photo could not be analysed for best shot (decode or metrics failed). | The others are still grouped; check that photo opens in the preview. |
| `ERR-IMG-0002` | v0.9.1 | `decode.failed` | `crates/core/src/codec.rs:45` | The photo could not be decoded | The decoder rejected the data: a truncated or damaged file, or a layout neither decoder handles (ADR-0033). | Open the file in another viewer; if it opens there, report it with the file (or its layout from the diagnostics). |
| `ERR-IMG-0003` | v0.9.1 | `import.suspicious` | `crates/core/src/import.rs:92` | The file looks suspicious and was skipped | The file's bytes do not match its extension or look crafted (magic-byte check). | Re-export the photo from its source; report it if it came straight from a camera. |
| `ERR-IMG-0004` | v0.9.1 | `import.too_large` | `crates/core/src/codec.rs:46` | The file is too large | The image is wider or taller than the decoder limit, or the file is over the size cap (decompression-bomb guard, SECURITY §3). | Use a smaller copy; if the photo is genuine, report it with its size. |
| `ERR-IMG-0005` | v0.9.1 | `import.unsupported` | `crates/core/src/codec.rs:50` | This format is not supported yet | The format is not one Fotollection reads (not JPEG, PNG, HEIC or a known RAW). | Convert it to JPEG, or open an issue with the file type. |

### RUS: Rust core

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-RUS-0001` | v0.9.1 | `culling.model_failed` | `crates/core/src/culling/mod.rs:49` | The face model could not run | The bundled face model could not run. | Best shot still works without faces; report it with the system. |
| `ERR-RUS-0002` | v0.9.1 | `encode.failed` | `crates/core/src/codec.rs:47` | The image could not be encoded | The JPEG encoder failed on an image the app made. | Report it with the diagnostics. |
| `ERR-RUS-0003` | v0.9.1 | `internal` | `src-tauri/src/state.rs (`IpcError::internal`)` | Something went wrong | An internal failure: a poisoned lock, a failed background task, or an unexpected state. The message carries the detail. | Report it with the diagnostics. |
| `ERR-RUS-0004` | v0.9.1 | `render.failed` | `crates/core/src/export.rs:50` | The preview could not be rendered | A render plan could not be executed (crop outside the image, resize failure). | Report it with the photo's size and crop. |

### HDW: Hardware: cameras, cards, drives

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-HDW-0001` | v0.9.1 | `camera.already_connected` | `src-tauri/src/camera.rs:749` | A camera is already connected | A camera is already connected. | Disconnect it first. |
| `ERR-HDW-0002` | v0.9.1 | `camera.busy` | `src-tauri/src/camera.rs:255` | Another app is using the camera. Close Imaging Edge Remote, then try again | Another app holds the camera, or a shot is still in progress. | Close Imaging Edge Remote, or wait for the shot. |
| `ERR-HDW-0003` | v0.9.1 | `camera.finishing` | `src-tauri/src/camera.rs:821` | (the Rust message) | The camera is still writing its last photo. | Try again in a moment. |
| `ERR-HDW-0004` | v0.9.1 | `camera.no_shot` | `src-tauri/src/camera.rs:344` | The camera did not take a photo: it may not have found focus. Try again | The camera did not take a photo, usually no focus. | Try again; check focus. |
| `ERR-HDW-0005` | v0.9.1 | `camera.not_connected` | `src-tauri/src/camera.rs:713` | No camera is connected | A capture command ran with no camera connected. | Connect the camera first. |
| `ERR-HDW-0006` | v0.9.1 | `camera.not_found` | `src-tauri/src/camera.rs:1376` | No camera found. Set USB Connection to PC Remote on the camera, check the cable, and close Imaging Edge Remote | No supported camera answered on USB. | Set USB Connection to PC Remote, check the cable, close Imaging Edge Remote. |
| `ERR-HDW-0007` | v0.9.1 | `camera.still_connected` | `src-tauri/src/camera.rs:924` | Close the capture screen first | The session is still open on the capture screen. | Close the capture screen first. |
| `ERR-HDW-0008` | v0.9.1 | `camera.timeout` | `src-tauri/src/camera.rs:233` | The camera did not answer. Check the cable and that the camera is on | The camera did not answer in time. | Check the cable and that the camera is on. |
| `ERR-HDW-0009` | v0.9.1 | `camera.unplugged` | `src-tauri/src/camera.rs:795` | The camera was unplugged or turned off | The camera was unplugged or turned off during the session. | Reconnect it. |

### DRV: Drivers: camera USB/PTP, WebKitGTK, WIC

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-DRV-0001` | v0.9.1 | `camera.bad_answer` | `src-tauri/src/camera.rs:267` | The camera's answer could not be read | The camera's PTP answer could not be parsed. | Report the model and firmware. |
| `ERR-DRV-0002` | v0.9.1 | `camera.failed` | `src-tauri/src/camera.rs:234` | The camera stopped answering. Check the cable | The camera stopped answering mid-command. | Check the cable; reconnect. |
| `ERR-DRV-0003` | v0.9.1 | `camera.refused` | `src-tauri/src/camera.rs:240` | The camera refused. Check that it is in PC Remote mode | The camera refused the PTP command. | Check it is in PC Remote mode. |
| `ERR-DRV-0004` | v0.9.1 | `camera.unavailable_platform` | `src-tauri/src/camera.rs:741` | Connecting a camera works on Windows for now | Camera capture has no driver on this system yet (Windows only for now, none on Linux, ADR-0034). | Use Windows for capture. |
| `ERR-DRV-0005` | v0.9.1 | `camera.unsupported` | `src-tauri/src/camera.rs:1414` | This camera is not supported yet: Sony bodies come first | The camera answered but is not supported (Sony bodies first). | Report the model. |

### EXT: Extensions and outside codecs

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-EXT-0001` | v0.9.1 | `import.heic_codec_missing` | `crates/core/src/codec.rs:48` | HEIC needs the free "HEIF Image Extensions" and the "HEVC Video Extensions" from the Microsoft Store | Windows has no HEIC decoder: the HEIF and HEVC extensions are missing. | Install both free extensions from the Microsoft Store, then refresh the folder. |
| `ERR-EXT-0002` | v0.9.1 | `import.heic_linux` | `crates/core/src/import.rs:93` | HEIC photos are not supported on Linux yet: convert them to JPEG, or open them on macOS or Windows | HEIC is not supported on Linux yet; FR-068 will use the system libheif. | Convert to JPEG, or open the photos on macOS or Windows. |
| `ERR-EXT-0003` | v0.9.1 | `import.raw_codec_missing` | `crates/core/src/codec.rs:49` | RAW photos need the free "Raw Image Extension" from the Microsoft Store | No reader could open this RAW and the system has no codec for it. **Unreachable since v0.8.0:** `raw.rs` `first()` swallows this error and reports `decode.failed` (ERR-IMG-0002); v0.9.1 either raises it again or retires it, and its message ("needs the Raw Image Extension") is out of date. | Report the camera model; installing Microsoft's Raw Image Extension may open it meanwhile. |

### FS: File system

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-FS-0001` | v0.9.1 | `camera.delete_failed` | `src-tauri/src/camera.rs:1022` | The photos could not be deleted | The session's photos could not be deleted. | Delete the session folder by hand if needed. |
| `ERR-FS-0002` | v0.9.1 | `camera.not_imported` | `src-tauri/src/camera.rs:1224` | The shot was saved in the session folder but could not be added to the tab | The shot was saved in the session folder but could not be added to the tab. | Refresh the session tab. |
| `ERR-FS-0003` | v0.9.1 | `camera.save_failed` | `src-tauri/src/camera.rs:991` | The photos could not be moved. Check the destination folder and try again | The session's photos could not be moved to the chosen folder. | Check the destination folder and try again. |
| `ERR-FS-0004` | v0.9.1 | `export.write_failed` | `crates/core/src/export.rs:51` | The file could not be written | An exported file could not be written: the destination is read-only, full or removed. | Choose another destination or free disk space. |
| `ERR-FS-0005` | v0.9.1 | `font.unreadable` | `src-tauri/src/commands.rs:1984` | The font file could not be read | The chosen font file could not be read. | Check the file still exists and can be opened. |
| `ERR-FS-0006` | v0.9.1 | `font.write_failed` | `src-tauri/src/commands.rs:2017` | The font could not be saved | The font could not be copied into the app's fonts folder. | Check free disk space and permissions of the app data folder. |
| `ERR-FS-0007` | v0.9.1 | `import.unreadable` | `crates/core/src/import.rs:91` | The file could not be read | The file could not be read: permissions, a removed card, or a cloud placeholder not downloaded. | Check the file opens in Finder/Explorer and that the card or drive is still there. |
| `ERR-FS-0008` | v0.9.1 | `io.read_failed` | `crates/core/src/culling/cache.rs:33` | (the Rust message) | The best-shot cache could not be read from disk. | Clear the cache in the versions popover; report it if it comes back. |
| `ERR-FS-0009` | v0.9.1 | `logo.unreadable` | `src-tauri/src/commands.rs:1325` | The logo file could not be read | The chosen logo file could not be read. | Check the file still exists and can be opened. |
| `ERR-FS-0010` | v0.9.1 | `logo.write_failed` | `src-tauri/src/commands.rs:1337` | The logo could not be saved | The logo could not be copied into the app's logos folder. | Check free disk space and permissions of the app data folder. |
| `ERR-FS-0011` | v0.9.1 | `project.rename_failed` | `src-tauri/src/document.rs:636` | The project could not be renamed | The project file could not be renamed. | Check the folder is writable and the file not open elsewhere. |
| `ERR-FS-0012` | v0.9.1 | `project.write_failed` | `crates/core/src/bundle.rs:98` | The project could not be saved | The project could not be written (atomic save failed). | Save to another folder; the previous save and its .bak are untouched. |

### PRJ: Project files (.fotollection)

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-PRJ-0001` | v0.9.1 | `project.corrupt` | `crates/core/src/bundle.rs:96` | The project file is damaged | The bundle or its project.json is damaged. | Open the .fotollection.bak next to it; report it with the app version. |
| `ERR-PRJ-0002` | v0.9.1 | `project.invalid_name` | `src-tauri/src/document.rs:611` | Use a name without / \\ : * ? " < > \| (up to 120 characters) | The new name has a character a file name cannot hold, or is too long. | Use a name without / \ : * ? " < > \|, up to 120 characters. |
| `ERR-PRJ-0003` | v0.9.1 | `project.name_taken` | `src-tauri/src/document.rs:631` | A file with this name already exists in the folder | A file with that name already exists in the folder. | Choose another name. |
| `ERR-PRJ-0004` | v0.9.1 | `project.newer_version` | `crates/core/src/bundle.rs:95` | This project was saved by a newer version of Fotollection | The project was saved by a newer Fotollection (schema too new). | Update Fotollection (Check for updates). |
| `ERR-PRJ-0005` | v0.9.1 | `project.not_a_project` | `crates/core/src/bundle.rs:94` | This is not a Fotollection project | The file is not a .fotollection bundle. | Open a file saved by Fotollection. |
| `ERR-PRJ-0006` | v0.9.1 | `project.too_large` | `crates/core/src/bundle.rs:97` | The project file is too large | The bundle is over the size cap (SECURITY §3). | Report it; projects should be far below the cap. |

### CFG: Settings, fonts and logos

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-CFG-0001` | v0.9.1 | `font.invalid` | `crates/core/src/fonts.rs:76` | This is not a valid font | The font file did not parse. | Choose another font. |
| `ERR-CFG-0002` | v0.9.1 | `font.missing_digits` | `crates/core/src/fonts.rs:77` | The font has no digits 0–9 | The font has no 0–9 digits, which the date stamp needs. | Choose a font with digits. |
| `ERR-CFG-0003` | v0.9.1 | `font.too_large` | `crates/core/src/fonts.rs:75` | The font file is larger than 20 MB | The font file is over 20 MB. | Choose a smaller font. |
| `ERR-CFG-0004` | v0.9.1 | `font.too_many` | `crates/core/src/bundle.rs:99` | You can import up to 50 fonts | The 50-font limit is reached. | Remove a font first. |
| `ERR-CFG-0005` | v0.9.1 | `font.unsupported` | `crates/core/src/fonts.rs:74` | Only OTF and TTF fonts are supported | The font is not OTF or TTF. | Choose an OTF or TTF file. |
| `ERR-CFG-0006` | v0.9.1 | `logo.decode_failed` | `crates/core/src/logo.rs:38` | The logo could not be decoded | The PNG did not decode. | Re-save the logo as PNG. |
| `ERR-CFG-0007` | v0.9.1 | `logo.not_png` | `crates/core/src/logo.rs:36` | The logo must be a PNG file | The logo is not a PNG. | Choose a PNG file. |
| `ERR-CFG-0008` | v0.9.1 | `logo.too_large` | `crates/core/src/logo.rs:37` | The logo is larger than 2 MB or 4096 px | The logo is over 2 MB or 4096 px. | Use a smaller PNG. |
| `ERR-CFG-0009` | v0.9.1 | `settings.invalid` | `src-tauri/src/commands.rs:436` | These settings are not valid | A setting from the UI was out of range or malformed (Rust validation). | Report it: the UI should never send it. |

### EXP: Export

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-EXP-0001` | v0.9.1 | `export.cancelled` | `crates/core/src/export.rs:52` | Export cancelled | The user cancelled the export (not a fault). | Nothing to do. |
| `ERR-EXP-0002` | v0.9.1 | `export.empty` | `src-tauri/src/commands.rs:666` | There are no photos to export | The view, or the ticked photos, hold nothing to export. | Show some photos, or tick some, then export. |
| `ERR-EXP-0003` | v0.9.1 | `export.no_destination` | `src-tauri/src/commands.rs:650` | Choose a destination first | Export started before a destination was chosen. | Choose a destination. |

### NET: Network (Check for updates only)

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-NET-0001` | v0.9.1 | `update.none` | `src-tauri/src/updates.rs:201` | No newer version was found. | No newer release than this one (information). | Nothing to do. |
| `ERR-NET-0002` | v0.9.1 | `update.offline` | `src-tauri/src/updates.rs:149` | Could not reach GitHub. Check your internet connection and try again. | Check for updates could not reach api.github.com. | Check the internet connection and try again. |
| `ERR-NET-0003` | v0.9.1 | `update.unreadable` | `src-tauri/src/updates.rs:180` | GitHub's answer could not be read. Try again later. | GitHub's answer was not the expected JSON or was over 1 MB. | Try again later. |

### IPC: Invalid input from the interface

| Code | Since | Rust code | Raised in | The user sees | Cause | What to do |
|------|-------|-----------|-----------|---------------|-------|------------|
| `ERR-IPC-0001` | v0.9.1 | `camera.bad_destination` | `src-tauri/src/camera.rs:964` | Choose a folder outside Fotollection's own data | The destination is inside the app's own data. | Choose another folder. |
| `ERR-IPC-0002` | v0.9.1 | `camera.bad_format` | `src-tauri/src/camera.rs:882` | Choose JPEG or RAW | A capture format other than JPEG or RAW. | Report it: the UI should never send it. |
| `ERR-IPC-0003` | v0.9.1 | `camera.bad_step` | `src-tauri/src/camera.rs:868` | A step is between −10 and 10 | A focus or exposure step outside −10…10. | Report it: the UI should never send it. |
| `ERR-IPC-0004` | v0.9.1 | `camera.bad_time_zone` | `src-tauri/src/commands.rs:1412` | The time zone is outside UTC−14…+14 | A time zone outside UTC−14…+14. | Pick a valid zone. |
| `ERR-IPC-0005` | v0.9.1 | `camera.destination_open` | `src-tauri/src/camera.rs:973` | That folder is already a tab: choose another one | The destination folder is already a tab. | Choose another folder. |
| `ERR-IPC-0006` | v0.9.1 | `camera.not_session` | `src-tauri/src/camera.rs:918` | This folder is not a camera session | The folder is not a camera session. | Pick a session tab. |
| `ERR-IPC-0007` | v0.9.1 | `camera.offset_too_large` | `src-tauri/src/commands.rs:1408` | A camera clock is corrected by at most a month | A clock offset over a month. | Enter a smaller correction. |
| `ERR-IPC-0008` | v0.9.1 | `camera.same_clock` | `src-tauri/src/commands.rs:1468` | Pick a photo from another camera or phone | The reference photo is from the same camera. | Pick a photo from another camera or phone. |
| `ERR-IPC-0009` | v0.9.1 | `camera.unknown` | `src-tauri/src/commands.rs:1286` | Unknown camera | A camera clock id that does not exist. | Refresh the camera clocks list. |
| `ERR-IPC-0010` | v0.9.1 | `culling.empty` | `src-tauri/src/culling.rs:49` | There are no photos to analyse | Best shot was asked with no photos. | Show some photos first. |
| `ERR-IPC-0011` | v0.9.1 | `culling.invalid_groups` | `src-tauri/src/culling.rs:139` | Choose between 1 and 5,000 groups | The group list was empty or over 5,000. | Run Find best shots again. |
| `ERR-IPC-0012` | v0.9.1 | `culling.unknown_group` | `src-tauri/src/culling.rs:151` | This group is out of date. Run Find best shots again. | The group is out of date. | Run Find best shots again. |
| `ERR-IPC-0013` | v0.9.1 | `folder.unknown` | `src-tauri/src/camera.rs:915` | (the Rust message) | The UI sent a folder id that no longer exists. | Refresh the library. |
| `ERR-IPC-0014` | v0.9.1 | `photo.invalid_date` | `src-tauri/src/places.rs:45` | (the Rust message) | A date outside 1826–2100. | Pick a date in range. |
| `ERR-IPC-0015` | v0.9.1 | `photo.invalid_rating` | `src-tauri/src/commands.rs:1803` | Ratings are 0 to 5 stars | A rating outside 0–5 stars. | Report it: the UI should never send it. |
| `ERR-IPC-0016` | v0.9.1 | `photo.invalid_selection` | `src-tauri/src/collage.rs:81` | Select between 1 and 10,000 photos | A selection was empty, over 10,000 photos, or listed a photo twice. | Select fewer photos. |
| `ERR-IPC-0017` | v0.9.1 | `photo.unknown` | `src-tauri/src/collage.rs:288` | Unknown photo | The UI sent a photo id that no longer exists (stale view). | Usually harmless after a refresh; report it if it repeats. |
| `ERR-IPC-0018` | v0.9.1 | `place.bad_window` | `src-tauri/src/places.rs:195` | (the Rust message) | The time window for borrowed places is outside 1–60 minutes. | Pick 1–60 minutes. |
| `ERR-IPC-0019` | v0.9.1 | `place.query_too_long` | `src-tauri/src/places.rs:35` | (the Rust message) | The place search text is too long. | Shorten it. |
| `ERR-IPC-0020` | v0.9.1 | `place.unknown` | `src-tauri/src/places.rs:83` | (the Rust message) | A place id not in the bundled places. | Search the place again. |
| `ERR-IPC-0021` | v0.9.1 | `project.unknown_folder` | `src-tauri/src/document.rs:667` | This folder is not missing | A relink was asked for a folder that is not missing. | Nothing to relink. |
