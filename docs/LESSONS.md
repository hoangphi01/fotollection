# Lessons — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-04
>
> What each debugging session taught us, so the same mistake is not made twice. Every bug that took more than one look to find gets an entry here in the same commit as its fix ([DOCS_GUIDE.md](DOCS_GUIDE.md) §2). Newest first. Each entry says what we saw, why it happened, the fix, and the rule to follow from now on. Performance-only lessons stay in [OPTIMIZATION.md](OPTIMIZATION.md) (for example §4.3.1, interactive work off background queues).

## How we debug

1. **Measure or observe before guessing.** The v0.6.0 plan assumed a 32 MP preview might be built; measuring first showed the preview was already capped at 1600 px, and the real gap was zoom ([RESULT.md §11](RESULT.md#11-preview-and-zoom-cost-v060)).
2. **Split the path and log both ends.** A temporary log on the UI side (what was asked) and one on the Rust side (what was answered, how long) tells which layer fails in one try. Name them `DEBUG-…` and remove them before the commit (`grep -rn DEBUG ui/src src-tauri/src` must be empty).
3. **Reproduce with the user's photo when the user sees it.** A synthetic test passed while the real app was soft: unit tests prove the pixels, only the real webview proves what is shown.
4. **Write the lesson here** before moving on.

## 2026-11-04

### A card of controls in the preview was spaced as a card of facts (FR-007)

- **Seen:** the owner found the preview's *Print / export size* card tighter than usual: its toggles, size picker, advice and border switch sat 4 px apart, against 12 px in the same card in the panel and in *Date stamp* beside it.
- **Why:** it was an `InspectorSection`, whose tight rows are meant for the photo's facts (capture date, location); the same controls in the panel sit in a `SectionCard`.
- **Fix:** the preview's print size card is a `SectionCard`; STYLE_RULE §4 says which card is for what.
- **Rule:** a card of controls is a `SectionCard` wherever it appears; `InspectorSection` is only for facts. Compare a shared card in both places side by side before calling it done.

### Measure the benchmark too: half the "memory after thumbnails" was the benchmark's own (NFR-004)

- **Seen:** the real trip's export peaked at 2.14 GB; 1.27 GB stayed resident after the thumbnail pass, before the export began.
- **Why:** `measure_folder` made T1 thumbnails on every core, where the app uses two background threads, and kept its own preview measurements (a decoded 20 MP photo, the file, a preview cache) alive through the export. The real cost was elsewhere: the print encoder's optimized Huffman tables keep every coefficient of the image, about 160 MB a 20 MP photo, and take a third of the export's time, for files 0.9 % smaller.
- **Fix:** the benchmark mirrors the app (two T1 threads, its measurements let go, the export's RSS sampled while it runs); prints use the standard tables; photos in flight follow the largest one.
- **Rule:** before optimizing a number, check that the tool measures what the app does, and time each stage of one item; a lossless option still has a cost in memory. Two JPEG encoder paths can differ by a few levels even when only the Huffman setting changes, so compare each against the source, not against each other.

### `glyph_bounds` is the line box, not the glyph: a "cap height" that was always 1 (FR-076)

- **Seen:** the test that keeps a bundled postcard face's strokes thick enough failed the same way after scaling the handwriting face by its capital height: the scale was exactly 1.
- **Why:** `postcard::cap_height` measured `font.glyph_bounds(&scaled_glyph('H'))`, which ab_glyph sizes by the face's ascent and descent, the same for every glyph. Every face got 1, so the stamp box's word was sized by its whole line, not by its capitals as its comment said.
- **Fix:** a capital's height now comes from its outline (`outline_glyph(…).px_bounds()`, `postcard::capital`). The old helper went: the stamp word is the same size as before, and its constant now says it is a line height.
- **Rule:** to measure ink, use the outline's `px_bounds`; `glyph_bounds` is only for laying out lines.

## 2026-11-03

### Deleting `target/debug/incremental` by hand can leave a debug build that does not link

- **Seen:** after freeing disk by removing `target/debug/incremental` (8 GB), `npm run dev` failed at link time: `Undefined symbols … drop_glue … rayon_core … fotollection_core.llvm.…`, while `cargo check` and the tests passed.
- **Why:** the core crate's `.rlib` came from a build that used the deleted incremental session; the next build reused it beside freshly compiled crates, and the internal (`.llvm.`) symbols no longer matched.
- **Fix:** `cargo clean -p fotollection-core` and build again (a minute, not a full rebuild).
- **Rule:** free disk with `scripts/clean.sh` (whole profiles), not by deleting part of a profile; if you did, clean the crates that failed to link.

### A setting that regroups the sheets must say so from Rust: the count stayed at 25 after turning postcards on (FR-073)

- **Seen:** the owner turned the new Postcard switch on in the dev app: each sheet showed one photo, but the bar still said 25 sheets for 50 photos, and the grid kept 25 tiles.
- **Why:** the sheet list (`useSheets`) reloads when the filter changes or Rust sends `LibraryChanged`. A setting is not a library edit, so nothing told it that pairing had changed. Listing again from the interface right after the switch would race `set_settings`: the interface shows the new value before Rust has stored it, and the two commands may run in either order.
- **Fix:** `set_settings` emits `LibraryChanged` after storing a settings change that turns postcards on or off, so every sheet list reloads from the new pairing.
- **Rule:** when a setting changes what a list contains, Rust announces it after the change is stored; the interface never guesses from its optimistic copy.

### Sampling at twice the output size still aliases past a 2× reduction: straightened photos got bands (FR-006, FR-072)

- **Seen:** a probe for session 6 (stripes 2 px apart, straightened by 0.5° and shrunk) gave flat grey up to 2.5× but bands at 3× and 4× (standard deviations of 44 and 91 levels, against about 1).
- **Why:** `straighten::render` sampled the turned crop at twice the output size, then finished with Lanczos3. At a 3× reduction that is one sample every 1.5 source pixels and at 4× every 2: the bicubic sample sees one pixel, not their average, and a pattern finer than the step folds into a coarse one. The comment said "so fine detail does not alias"; it held only up to 2×.
- **Fix:** sample at the crop's own resolution (one sample per source pixel at most), then Lanczos3 (`fr_072_straightened_reductions_do_not_alias`). The preview samples its 2048 px source as before; only a share export of a straightened photo samples more.
- **Rule:** a resampling claim gets a test with a pattern at the Nyquist limit over the whole range of factors, not one factor.

### png 0.18 fills `source_gamma` only when encoding: a PNG's `gAMA` read as absent (FR-072)

- **Seen:** the new test `fr_072_png_gamma_is_converted` wrote a PNG with `gAMA` 1.0 (linear light) and expected sRGB 188 for 128; it came back 128, unconverted. `colour::declared_to_srgb` alone gave 188.
- **Why:** after `read_info`, png 0.18's `Info::source_gamma` and `source_chromaticities` stay `None`; the chunks as read are in `gama_chunk` and `chrm_chunk` (`source_*` is what the encoder writes, and the docs only say "prefer `source_gamma`").
- **Fix:** `decode_png` reads `gama_chunk` and `chrm_chunk`; `srgb` still marks a file as sRGB.
- **Rule:** for a decoder's metadata field, write a file with the encoder and read it back in a test before trusting the field's name.

### A lock held across rayon work hangs the pool: zooming into a RAW froze every preview (FR-057, FR-016)

- **Seen:** the owner opened an ARW, zoomed (the sharp layer came slowly), pressed → for the next photo: its preview spun for five minutes, and so did every photo opened after closing the preview. `measure_zoom` on its own was fast (preview 0.9 s, first region 0.7 s), but on a two-thread pool with a region asked every 150 ms, as the preview window asks, three DNGs stopped after the first photo with the CPU idle: a deadlock, not slowness. The Mac's ARW path (ImageIO) uses no rayon and hid it; rawler (Windows) and our DNG decoder use rayon inside the develop.
- **Why:** `raw::decode` took the `DEVELOP` mutex (one develop at a time, NFR-004) on the pool thread, then the develop ran `par_iter`. A rayon thread that waits for its parallel work runs other queued jobs of its pool meanwhile; it took up a zoom request, which called `raw::decode` and waited for `DEVELOP`, held below it on the same thread. The other thread waited for the lock too, and no preview, region or source could be answered again. A batch export of RAWs could hang the same way. And the first sharp region developed the RAW a second time, because the preview kept only its 2048 px copy.
- **Fix:** `raw::one_at_a_time` runs the develop and its lock on a plain thread of its own, which never takes up other jobs; the caller only waits (test `develops_asked_while_one_runs_all_finish` hangs on the old code). The preview keeps the full image it developed in the zoom cache (`PreviewCache::keep_full`), so the first region needs no second develop (ARW 712 → 42 ms, OPTIMIZATION §5). A failed preview answers its `IpcError` as JSON and the stage shows it with its `ERR-` code; before, an image that failed to load left the spinner on for good.
- **Rule:** never hold a lock while rayon work runs (STYLE_RULE §2, Errors). Test what runs on a pool on a small pool, with requests arriving while one runs, not one call at a time. And every image the UI waits for has an `onerror`.

## 2026-11-02

### A crate yanked between CI and the release stops the release (v0.9.1)

- **Seen:** the push of `chore(release): v0.9.1` failed its `security` check: `cargo deny` found `async-executor 1.14.0` (under zbus, on Linux) yanked. The same lockfile had passed CI an hour earlier. Pushing the fix in `Cargo.lock` started no release, because only a change to `Cargo.toml` does.
- **Why:** a yank lands at any time and `cargo deny` checks against today's index. `scripts/release.sh` runs `cargo update --workspace`, which updates only our own crates.
- **Fix:** `cargo update -p async-executor` (1.14.1), committed, then the `release` workflow started by hand (`gh workflow run release --ref main`). No tag had been made, so nothing was published twice.
- **Rule:** run `cargo deny check advisories` just before `scripts/release.sh`; when a release fails before *publish*, fix on `main` and start the workflow by hand, never bump the version for it.

### A repeating pattern has an origin: DNG counted the CFA and black level from the wrong corner (ADR-0030)

- **Seen:** reading `rawcore/src/cfa.rs` against the DNG specification, `site` and `level` took the 2×2 phase from the stored image's `(x & 1, y & 1)`. A synthetic DNG with its `ActiveArea` at row 1, column 1 read red as green and each pixel with its neighbour's black level; no sample had shown it, because nearly every camera starts its active area on an even row and column.
- **Why:** DNG starts `CFAPattern` and `BlackLevel` at the active area's top-left corner. The code was written and tested only on areas at (0, 0), where both origins agree.
- **Fix:** `Mosaic::phase` counts from `active[0]`, `active[1]`; `BlackLevelRepeatDim` 1×2 and 2×1 are read as well.
- **Rule:** every pattern, table or offset in a format names its origin; write the test with the origin moved off zero (odd, for a 2×2 pattern), where a wrong origin cannot hide.

### A fixture that cannot fail proves nothing: the 16-bit grey test passed on broken code (ERR-IMG-0002, ADR-0033)

- **Seen:** the audit read that the 16-bit grey path keeps the wrong byte of each sample (jpeg-decoder gives them in the machine's byte order). The first fixture for it decoded with no error at all on the old code.
- **Why:** the fixture's samples were `luma × 257`, whose high and low bytes are equal, so keeping either byte gives the same picture. A second fixture, with a low byte unrelated to the high one, failed on the old code at once (mean error 79 of 255), and passes with the fix.
- **Also found by writing fixtures first:** a JPEG cut short decoded "fine" as half a photo on grey (now refused, `ERR-IMG-0006`); Adobe CMYK, suspected, decodes right on zune, while the reference decoder's fallback really came out negative (a difference of 255 between the two decoders).
- **Rule:** before trusting a fixture, make it fail on the code it is meant to catch: build it so the wrong answer and the right one differ, and run it once on the old code. Read the dependency's source for byte order and inversions, never the comment above our call.

## 2026-11-01

### Rotate photo turned the whole collage sheet (FR-035)

- **Seen:** on a collage sheet, *Rotate photo* on the first photo stood the sheet up instead of turning only the photo in its half.
- **Why:** the sheet takes its orientation from its first photo's shape, and that shape was read after the photo's turns in its cell, so turning the photo also changed the input that decides the sheet.
- **Fix:** `sheet::layout_turned` lays the sheet out from the photo before its cell turns, in the panel, the crop editor, the preview and the export.
- **Rule:** a setting that changes one thing must not feed the input of another decision; when a value is derived (the sheet's orientation from the first photo), derive it from the state before the new edit, and test that the edit leaves the rest unchanged.

### The APT repository's first real run copied a folder into itself (ADR-0035)

- **Seen:** the first website build after v0.8.2 failed in *APT repository*: `cp: cannot copy a directory, '.', into itself, '_site/apt/.'`. Every earlier build had skipped the step, since no release had a `.deb` yet.
- **Why:** `apt-repo.sh` took its output folder as given (`_site/apt`, relative), then `cd` into its temporary folder to build; the relative path then pointed inside that folder.
- **Fix:** the script makes the output path absolute before it changes folder.
- **Rule:** a script that changes folder resolves its path arguments first. A step that only runs once a release exists is run by hand before that release (here, against a test release or a local `.deb`), or its first real run is the test.

### A bare container is not a desktop: the first v0.8.2 run stopped on its own smoke test (ADR-0034)

- **Seen:** the arm64 build passed on Ubuntu 22.04, then the AppImage exited with 127 in `debian:12`: `libfontconfig.so.1: cannot open shared object file`. The failed job cancelled the other three, so nothing was released.
- **Why:** an AppImage leaves out on purpose the libraries every desktop already has (fontconfig, freetype, GL, X11: the AppImage exclude list), because bundling them breaks on other drivers. The `debian:12` image has none of them; no one runs a photo app there.
- **Second run:** `ldd` found nothing missing, then WebKit aborted on `libGLESv2.so.2`: it opens GLES with `dlopen` at run time, which `ldd` cannot see. Every desktop has it through Mesa.
- **Fix:** the container first installs GTK 3 and Mesa (`libgtk-3-0 libgl1 libegl1 libgles2 libgbm1 libgl1-mesa-dri`), the floor of any Debian desktop, then runs `ldd` over the extracted AppImage and prints every library still missing before it starts the app. The installers are uploaded before the smoke test and the build matrix no longer stops at the first failure, so a failed run leaves every installer to try locally (Docker on the M3 runs the arm64 one) instead of costing another 30-minute run.
- **Rule:** a test environment must match what a user has, no less: test an AppImage on the desktop floor, not on a bare image. A smoke test that fails prints everything that is wrong at once, and keeps what it tested; a library check by `ldd` misses what is opened at run time, so the app must still start. A release dry run (dispatch before the version bump) catches this without a failed release run.

### Typing in the date stamp made the panel jump and the text slip back (FR-036)

- **Seen:** each keystroke in *Line under the date* made the cards below jump up, and the text sometimes went back a few letters before catching up.
- **Why:** two faults. `useSheetInfo` returned `null` whenever `revision` changed, so the note under *Two photos per sheet* vanished until Rust answered and the card shrank. And `CommitInput` commits while typing (every 300 ms); `useRenderSettings` applied every answer, so a late answer to an earlier commit put the older text back into the field.
- **Rule:** keep the last answer while a new one loads (`useLiveQuery`); apply only the answer to the latest request; never feed an outside value into a focused field. STYLE_RULE §4, "Nothing blinks, nothing jumps".

### The dev app failed to link after two builds overlapped (tooling)

- **Seen:** `npm run dev` rebuilt on a file change while `cargo test` was running, and then every start failed with `Undefined symbols … fotollection_core.llvm…` from the linker; a restart printed `Port 5173 is already in use`.
- **Why:** an interrupted build left `fotollection-core`'s incremental objects half written, so the binary linked against code that was not there. Stopping the Tauri process leaves its Vite server running.
- **Rule:** while the dev app runs, wait for its rebuild before running `cargo test` or clippy, or stop it first. On `Undefined symbols` in our own crate, `cargo clean -p fotollection-core` (it also freed 18 GB) and start again; before a restart, free the port with `lsof -ti :5173 | xargs kill`.

### Collage mode froze the window after one click (FR-048)

- **Seen:** with the audit's "Simple and Collage ignore Pro's star and flag filters", the Collage screen drew once, then stopped answering: no click, no key, the screenshot script timed out.
- **Why:** `app.tsx` built the filter for Simple and Collage (`{ ...filter, flag: 'all', minRating: 0 }`) anew on every render, and `useLibrary` refetched whenever the filter *object* changed. Each answer re-rendered the app, which built a new object, which refetched: an endless loop.
- **Fix:** `useLibrary` keys its effect on the filter's content (`JSON.stringify`), as `useSheets` already did.
- **Rule:** a hook that fetches from IPC keys its effect on the request's content (a string), never on an object a caller may rebuild.

### A decoder can be silently wrong: JPEGs came out as green stripes (ADR-0033)

- **Seen:** a v0.8.0 user's folder showed half its photos as green tiles with fine horizontal stripes and a ghost of the picture. The owner first suspected our RAW decoder; the files were JPEGs.
- **Why:** they were saved with one scan per component and colour halved both ways (4:2:0), a rare but valid baseline layout. `zune-jpeg` 0.5.15 reads the Cb scan into the Cr one and returns the result as a success: the brightness survives as the ghost, the colour planes are wrong, hence green. The same decode feeds thumbnails, preview, best shot and export, so prints were green too, and the bad thumbnails stayed in the disk cache keyed only by the file.
- **How we found it:** the symptom (luma kept, chroma wrong, in rows) pointed at the colour planes of a JPEG decode; the upstream tracker had the layout; one fixture per layout, saved with `cjpeg`, reproduced it (mean error 111.9 against 2–4 for the others) before any fix.
- **Fix:** read the layout from the header (`jpeg_layout`); zune decodes only the layouts the corpus proves, `jpeg-decoder` every other one and any file zune refuses; `thumbs::CACHE_VERSION` makes every thumbnail and analysis again.
- **Rule:** a fast decoder is trusted only on the layouts a test proves; everything else goes to a reference decoder. Every decoder upgrade runs the layout corpus first, and an RC never ships. A fix that changes the pixels of files already seen bumps the cache version. When a user reports a picture problem without the file, rebuild the layout from the symptom and make a fixture of it.

## 2026-10-31

### The website build broke on a Liquid include (landing page, FR-048)

- **Seen:** the Pages build failed with "Invalid syntax for include tag: name=r alt=t[show_key]" after the "Three modes" section was added; the live site kept the previous build.
- **Why:** Jekyll's `include` parameters take a variable or a quoted string, not a lookup like `t[key]`. The print-basket section next to it assigns the lookup to a variable first, and the new section was written without that step.
- **Fix:** `assign show_alt = t[show_key]`, then `alt=show_alt`, and a named `cycle` so the two sections keep their own alternation.
- **Rule:** never pass `t[...]` straight to `include`; copy the nearest working section's Liquid, and watch the `pages` run after every site push.

### Simple mode came up in English (FR-048)

- **Seen:** on a Vietnamese machine, the Simple mode screen and its panel were in English, while Professional was in Vietnamese.
- **Why:** the saved language was applied by `useLanguage`'s effect, and the only component that called it was the language card in Professional's settings panel. Simple mode does not mount that panel, so the language was never applied.
- **Fix:** `App` calls `useLanguage()`, so the language applies in every mode.
- **Rule:** a per-machine preference is applied once, at the top of the app, never as a side effect of the control that changes it.

## 2026-10-27

### Our own DNG was 1 ΔE off the camera's JPEG because of the lens, not the colour (ADR-0030)

- **Seen:** with the DNG colour model, one tone curve and a fitted 3×3 matrix, rawcore stayed at ΔE 3.8 from the embedded JPEG, and more matrix or curve freedom did not help.
- **Why:** an error map by region showed it: the centre about 4 L\* too bright, the corners 1.5 too dark. The camera corrects its lens's vignetting in the JPEG; the raw data does not. A global curve cannot fix an error that depends on the position.
- **Fix:** a radial gain `exp(a·r² + b·r⁴)` fitted from the JPEG, alternating with the curve fit; ΔE 2.67, every region within ±0.5 L\*.
- **Rule:** when a global fit stops improving, map the error by region before adding parameters.

### TESTING.md described fuzz targets that had never been made

- **Seen:** the RAW quality gate asked for fuzzing, and TESTING.md listed four targets under `crates/core/fuzz/`. The folder does not exist, and the machine has no nightly toolchain or cargo-fuzz.
- **Why:** the testing plan was written before the code, as if it were already done, and nothing checked it.
- **Fix:** mutation testing on stable Rust (`crates/rawcore/examples/mutate.rs`); TESTING.md now says which fuzzing exists and which is planned.
- **Rule:** docs say "planned" for what is not built yet; check that a path a doc names exists before relying on it.

## 2026-10-26

### "Supported" was written after testing on one system only (FR-016)

- **Seen:** the website, the README's feature list and the v0.7.0 notes said camera RAW opens as it is, with nothing to install. On Windows it needs Microsoft's Raw Image Extension, and without it RAW files are skipped. The owner found it, not a test.
- **Why:** RAW was measured and checked on macOS, where ImageIO decodes it; the Windows path (WIC) was only described in one row of the README's requirements, while the claims were written as if they held everywhere. And the decision to decode RAW only through the OS (ADR-0023) was taken to keep the relicensing option, without asking the owner whether that was worth an extra install for every Windows user.
- **Fix:** the claims now say what Windows needs; a built-in RAW reader is planned (rawler now, our own decoder next).
- **Rules:** never write "supports" or "works" for a feature before it has been tried on every system the app ships for; and a constraint that costs users a feature (a licence rule, a platform limit) is the owner's decision, so ask before choosing it.

### `URL.pathname` is not a file path on Windows

- **Seen:** `npm run lint` failed on Windows because `check-i18n.mjs` opened `C:\C:\Users\…\en.json`.
- **Why:** `new URL('../src', import.meta.url).pathname` gives `/C:/Users/…` on Windows, and `path.join` then repeats the drive letter. CI runs on macOS and Linux, so it never showed there.
- **Fix:** `fileURLToPath(new URL(…))`.
- **Rule:** in Node scripts, turn a `file:` URL into a path with `fileURLToPath`, never `.pathname`.

### WPD hands over a maker's event code in the high half of the GUID (FR-063)

- **Seen:** the a6400 driven through WPD sent no events we recognised. A temporary log of each event's GUID showed `C2030000-5738-4FF2-8445-BE3126691059`, where the code was expected in the low 16 bits.
- **Why:** WPD builds a maker's event GUID from `WPD_EVENT_MTP_VENDOR_EXTENDED_EVENTS` and puts the PTP event code in the high 16 bits of its first field.
- **Fix:** `vendor_event_code` reads `data1 >> 16`; a test uses the GUID as seen.
- **Rule:** when a platform API rewraps camera data, log what it really hands over before trusting the documented shape.

### A Sony shot needs time between the shutter steps (FR-063)

- **Seen:** the four shutter steps, sent back to back (about 40 ms), answered `OK` but the a6400 did not shoot. No event came, and the shot state stayed 0.
- **Why:** the half press starts autofocus, and releasing within milliseconds cancels it. Remote's own fast presses also failed twice in the second recording.
- **Fix:** `SHUTTER_STEP_GAP` = 300 ms between steps; it shot 4 of 4. A missing `0xD215 = 0x8001` within a few seconds means "did not shoot".
- **Rule:** an `OK` to a control operation means "received", not "done". Check the camera's state afterwards.

### dumpcap does not see USBPcap (FR-063)

- **Seen:** on Windows with Wireshark, USBPcap and Npcap installed and the USBPcap driver running, `dumpcap -D` listed only network cards, so `scripts/camera-capture.ps1` said "No USBPcap interface found"; `tshark -D` listed `\\.\USBPcap1` and `\\.\USBPcap2`.
- **Why:** USBPcap reaches Wireshark as an extcap interface, and only `tshark` and Wireshark start extcap programs; `dumpcap` alone never lists or opens them.
- **Fix:** the script lists and records with `tshark`. When USBPcap is installed separately (winget), `USBPcapCMD.exe` must be copied into Wireshark's `extcap` folder, and a silent Wireshark install leaves out Npcap ([cameras/capture-guide.md §5](cameras/capture-guide.md#5-if-something-goes-wrong)).
- **Rule:** record USB on Windows through `tshark`, never through `dumpcap` directly.

### A camera says nothing while its card is being listed (FR-063)

- **Seen:** the GX9 opened a session, then did not answer `GetDeviceInfo` within 10 s; the a6400, with few photos, answered at once.
- **Why:** right after a session opens, ImageCaptureCore lists every file on the card (70 here) over the same USB pipe, and the body handles one transaction at a time, so our command waited behind the listing.
- **Fix:** `probe_camera --settle` waits before the first command; the real driver waits for `deviceDidBecomeReadyWithCompleteContentCatalog` before it sends anything, and keeps generous timeouts.
- **Rule:** with ImageCaptureCore, send nothing to a camera until its catalog is complete; a timeout right after opening is not proof that the camera cannot do the operation.

## 2026-10-25

### Installer settings fail only when installers are built (FR-054)

- **Seen:** the v0.7.0 release failed on Windows; v0.7.0-beta.1 had passed, because a pre-release builds no MSI.
- **Why:** the WiX fragment for "Open with" referenced the executable as `[#Path]`, a file in another feature of Tauri's template, which the MSI validator rejects (ICE69). `ci` compiles and tests but never bundles, so nothing ran this file before the release.
- **Fix:** the registry values name the executable by path (`[INSTALLDIR]fotollection.exe`); a new `installers` workflow builds the MSI and the NSIS installer whenever their settings change, and the release was started again by hand.
- **Rule:** anything that only the bundler reads (`src-tauri/windows/`, `tauri.conf.json`) is proved by the `installers` workflow before a release, not by the release.

### macOS hands over a file before the app exists (FR-054)

- **Seen:** opening a photo with Fotollection while it was closed started nothing: no window, no crash report. Run from the terminal, the bundle worked.
- **Why:** on a cold launch from Finder, macOS sends `RunEvent::Opened` before `setup` has called `manage(AppState)`. Routing the file called `app.state()`, which panics before `manage`, and the release build (`panic = "abort"`) died silently. Double-clicking a closed project hit the same path ever since FR-023. A second gap: the project request was announced only by `MenuAction`, which can fire before the page listens.
- **Found by:** `open --stdout … --stderr … -a Fotollection.app photo`, which shows the panic that the GUI launch hides.
- **Fix:** `open_with::route` queues paths while `try_state::<AppState>()` is `None` and setup routes them once start-up is over; the UI asks `document_requested` / `take_photo_request` after `StartupFinished` instead of relying only on events.
- **Rule:** an OS event handler never assumes the app state exists (`try_state`), and every OS hand-over is kept in Rust until the UI takes it. Test cold launches through `open -a`, not only with the app running.

### EXIF's Make is a company, not a brand (FR-040)

- **Seen:** the info frame said "Shot on NIKON CORPORATION NIKON D750" for the Nikon RAW sample.
- **Why:** the camera label dropped the make only when the model started with the whole make; Nikon, Ricoh and Olympus write the company ("NIKON CORPORATION", "RICOH IMAGING COMPANY, LTD.") in Make and the brand in Model.
- **Fix:** trailing company words are dropped from the make, and the model is compared word by word, ignoring case (`Library::camera_label`).
- **Rule:** text built from EXIF is checked on files from several makers, not only the owner's own camera; the CC0 RAW samples cover nine.

### A RAW can look like a plain TIFF (FR-016)

- **Seen:** measured before writing the feature, ImageIO returned a NEF as 160 × 120 and could not open an ARW at all, while `sips` handled both.
- **Why:** NEF, ARW, CR2 and DNG start with an ordinary TIFF header. From bytes alone ImageIO read the NEF as a TIFF (its first image is a thumbnail); `sips` passes the file name, so it knew.
- **Fix:** pass the format's type identifier as a hint, take the largest image in the file; on Windows ask WIC for the RAW decoder directly.
- **Rule:** measure every format on a real sample before building on it; a format family can hide a case that breaks the design.

### Some characters turn into emoji on iOS (website)

- **Seen:** the footnotes' back link `↩` and the `☕` in the support heading showed as colour emoji on an iPhone.
- **Why:** characters with an emoji presentation are drawn as emoji by iOS unless a text selector follows them.
- **Fix:** Lucide icons (`corner-left-up`, `coffee`) as inline SVG, like every other icon on the site.
- **Rule:** no pictographic characters in the UI or on the site; use Lucide (STYLE_RULE: Lucide icons only).

### A red check hid for three pushes (CI)

- **Seen:** the owner noticed red crosses on the last commits on GitHub; the `docs` workflow had failed since the v0.6.0 plan change, while only `pages` and `release` were being watched.
- **Why:** moving the releases added an item to the "Next, in order" list in STATUS.md without renumbering the last one (`7.` twice); markdownlint's MD029 rule fails on that. `check-docs.sh` does not run markdownlint, and the markdownlint line of DOCS_GUIDE §3 was skipped.
- **Fix:** renumbered; each workflow now runs only when its own files change (RELEASE.md §4), so every red cross means something.
- **Rule:** run every line of DOCS_GUIDE §3 before pushing docs.
- **It happened again** the same evening (`` `crx ` `` in ADR-0023 slipped through a `npx … | tail -1 && git commit` chain), so the fix became structural: `scripts/check-docs.sh` now runs markdownlint itself, with CI's version and rules, and its exit code is the only thing to check. After pushing, look at every run that started (`gh run list --limit 5`), not only the one expected.

### A CSS `scale()` throws image pixels away (FR-057)

- **Seen:** zoomed to 800 %, the full-resolution layer arrived (Rust log: 1,222 px wide, 45–137 ms) and sat in the right place, yet looked as soft as the 1600 px preview under it.
- **Why:** the layer was laid out at its small fit size (125 CSS px) and enlarged with `transform: scale(8)`. WebKit rasterises an element at its layout size first, then scales that bitmap, so the extra pixels never reach the screen.
- **Fix:** size the layer at its final on-screen size and only `translate` it (`use-sharp-region.ts`, `layerStyle`).
- **Rule:** to show detail, an image's layout box must have as many CSS pixels as it should display. CSS `scale()` is fine for motion, never for sharpness.

### Jekyll `include` parameters cannot be expressions (website)

- **Seen:** the live site still lacked the print-basket section; the GitHub Pages workflow had failed on every push since that commit, unnoticed.
- **Why:** `{% include shot.html alt=t[show_key] %}`: an `include` parameter must be a plain variable or a string. Liquid stops the whole build.
- **Fix:** `{% assign show_alt = t[show_key] %}` first, then `alt=show_alt`.
- **Rule:** after a website change, build it locally with the Pages image ([site/README.md](../site/README.md) "Preview locally") and check that the `pages.yml` run is green after the push.

### `command | tail` hides the exit code

- **Seen:** `scripts/check-docs.sh 2>&1 | tail -1 && git commit …` committed although the check had found two problems.
- **Why:** a pipeline returns the exit code of its last command, here `tail`, which succeeds.
- **Rule:** never chain on a piped check. Run the check on its own, or test its output (`grep -q "all checks passed"`), or use `set -o pipefail`.

### Editing code while a build runs produces a mixed build

- **Seen:** `tauri build` failed with "no `Region` in `preview`" although the code was complete.
- **Why:** Cargo had already compiled `fotollection-core` from the old `preview.rs` when the new code reached `src-tauri`.
- **Rule:** finish the edits, then build. While a long build runs, only edit files it does not compile (docs, the website).

### A hot reload that adds a hook to an open component blanks the app

- **Seen:** the dev app turned white after a temporary `useEffect` was added to the zoom hook while the preview was open.
- **Why:** React Fast Refresh keeps the mounted component's hook list; a different number of hooks is a hook-order error, and with no error boundary the tree unmounts.
- **Rule:** this is a dev-only artefact, not a bug: restart `npm run dev`. Prefer changes that keep the hook count when testing live.

### A screenshot of the start-up screen (website screenshots)

- **Seen:** the manual's frame-card shot came out as the logo and a loading bar, and a full shoot took almost 8 minutes instead of under 2.
- **Why:** the mock IPC bridge in `shoot.mjs` did not answer `startup_finished`, so the start-up screen (FR-060) stayed until its 15-second safety timer; shots taken right after loading caught it. `scrollIntoView` was not the cause, though it also scrolls the page, not only the panel.
- **Rule:** when the app gains a command it calls at start-up, answer it in the mock bridge; `open()` now waits until `#splash` is gone, and a shot that needs a scrolled panel scrolls that panel's own box.

### A narrow column left on its own looks broken (manual)

- **Seen:** the owner found the manual "pushed to one side": its text had `max-width: 880px` inside the 1180 px `.wrap`, left-aligned, so a wide empty band ran down the right.
- **Why:** reading width was chosen for the text alone, without comparing the page with the landing page, whose sections fill the `.wrap`.
- **Rule:** on the website, content spans the `.wrap` like the landing page's sections ([STYLE_RULE.md §4](STYLE_RULE.md#website-site)); measure that the left and right margins match before pushing.

### A comment in a heading changes its anchor (manual, ADR-0031)

- **Seen:** the docs workflow failed after the manual's chapter anchors went into the guide's titles (`## 1. Choosing the photos to print <!-- choose -->`): links from PRD and STATUS to `USER_GUIDE.md#1-choosing-the-photos-to-print` could no longer be found.
- **Why:** lychee builds a heading's anchor from its raw text, comment included, although GitHub does not show the comment. `check-docs.sh` does not check anchors, so only CI saw it.
- **Rule:** keep markers out of headings: put them on their own line above (`<!-- chapter: choose -->`). After renaming or marking a heading, `grep` the docs for links to its old anchor before pushing.

### GPL-3.0 cannot forbid sharing or selling (website)

- **Seen:** the owner asked for "no redistribution or resale in any form" on the site.
- **Why:** GPL-3.0 grants both and §7/§10 void any added restriction; such a line would mislead.
- **Fix:** the footnotes protect what can be protected: the official name, logo and downloads, a request to tell the author, a request to credit fotollection.org. Real options are relicensing future versions ([ADR-0008](adr/0008-license-gpl3-with-relicensing-option.md)) or a registered trademark.
- **Rule:** never publish a licence term the licence itself does not allow; say what actually protects the author.

## Earlier

### Rounding a zoom region can step outside the photo (FR-057)

- **Why:** rounding fractions to 4 places can make `x + w` slightly more than 1, which Rust rightly refuses.
- **Rule:** round region starts and sizes **down** in the UI; Rust still validates every value ([SECURITY.md](SECURITY.md) §2).

### Mock data falls behind the real bridge (screenshots)

- **Seen:** website screenshots showed `NaN` counts and settings that did not stick.
- **Why:** the mock IPC bridge in `site/screenshots/` lacked fields added to `PhotoList` and did not store `set_settings`.
- **Rule:** when a command or a type gains a field, update the mock bridge in the same commit, and read the shots before publishing them.

### Ask before building a layout the user described in words (FR-040)

- **Seen:** *Fit* was built twice the wrong way before the owner explained it with real sizes.
- **Rule:** for a visual rule, restate it with numbers (10 × 15 cm keeps 10 × 15; *Fit* adds 5 mm on three sides plus the bar) and get a yes before coding.

### Screenshot steps that pick photos by position (screenshots)

- **Seen:** after sixteen Prague photos joined the demo set, `shoot.mjs` timed out waiting for the crop photo's tile, then for a fifth photo in a folder tab.
- **Why:** the basket and set-aside photos were chosen by their position in the list (`TRIP_QUEUES`), so a new set put the crop photo in *Set aside*, out of the gallery; the folder tabs were matched by name and kept fewer photos than the clicks assumed. The gallery is virtualized too, so a tile far down the list is not in the page until it is scrolled to.
- **Rule:** when the demo set changes, check that every position still names a photo the shot does not show, and reach a photo's tile with `reach()` (it scrolls the grid) instead of assuming it is drawn. Screenshot the page when a wait times out; the cause is on screen.

### Faux bold by drawing twice doubles a thin face (FR-073)

- **Seen:** the owner's postcard showed bold "Minh" with a doubled M, two thin strokes side by side.
- **Why:** bold was the text drawn a second time 0.045 em to the right. Xanh Mono's strokes are thinner than that offset at print size, so the two copies did not touch.
- **Rule:** to embolden, draw at steps of at most half a pixel across the extra width and keep each pixel's highest coverage, then blend once (`text::draw_text_bold`); a test checks one run per stem. Look at a print-size render of a thin face before trusting a style.

### A fit check must count only what prints (FR-073)

- **Seen:** a postcard address that printed whole on its four lines still showed the red "does not fit" mark.
- **Why:** the text ended with a line break, and `wrap` turned it into one more, empty row, which printed nothing but counted as a fifth line. Rows past the last line were also measured with no room, so each left-over letter became a row of its own.
- **Rule:** drop trailing blank rows before counting, measure rows past the last line as long as it, and test the check on text as the app's field sends it (doubled spaces, a trailing break), not only on tidy strings.

### A lazy image with no height moves the anchors below it (website)

- **Seen:** the navbar's Coffee button scrolled to a point 800 to 1,800 px above the support section; every other anchor landed right.
- **Why:** the benchmark charts above it were `loading="lazy"` with a width and no height. The page scrolled toward `#support` with each chart counted at zero height; as they loaded on the way, the section moved down and the scroll stopped short.
- **Rule:** every lazy image on the site carries both `width` and `height` (charts get theirs from `site/_data/charts.json`, written by `bench/chart.mjs`). After a layout change, click every in-page link and check where it lands, not only that it scrolls.
