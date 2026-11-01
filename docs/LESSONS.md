# Lessons — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-01
>
> What each debugging session taught us, so the same mistake is not made twice. Every bug that took more than one look to find gets an entry here in the same commit as its fix ([DOCS_GUIDE.md](DOCS_GUIDE.md) §2). Newest first. Each entry says what we saw, why it happened, the fix, and the rule to follow from now on. Performance-only lessons stay in [OPTIMIZATION.md](OPTIMIZATION.md) (for example §4.3.1, interactive work off background queues).

## How we debug

1. **Measure or observe before guessing.** The v0.6.0 plan assumed a 32 MP preview might be built; measuring first showed the preview was already capped at 1600 px, and the real gap was zoom ([RESULT.md §11](RESULT.md#11-preview-and-zoom-cost-v060)).
2. **Split the path and log both ends.** A temporary log on the UI side (what was asked) and one on the Rust side (what was answered, how long) tells which layer fails in one try. Name them `DEBUG-…` and remove them before the commit (`grep -rn DEBUG ui/src src-tauri/src` must be empty).
3. **Reproduce with the user's photo when the user sees it.** A synthetic test passed while the real app was soft: unit tests prove the pixels, only the real webview proves what is shown.
4. **Write the lesson here** before moving on.

## 2026-11-01

### A bare container is not a desktop: the first v0.8.2 run stopped on its own smoke test (ADR-0034)

- **Seen:** the arm64 build passed on Ubuntu 22.04, then the AppImage exited with 127 in `debian:12`: `libfontconfig.so.1: cannot open shared object file`. The failed job cancelled the other three, so nothing was released.
- **Why:** an AppImage leaves out on purpose the libraries every desktop already has (fontconfig, freetype, GL, X11: the AppImage exclude list), because bundling them breaks on other drivers. The `debian:12` image has none of them; no one runs a photo app there.
- **Fix:** the container first installs GTK 3 and GL (`libgtk-3-0 libgl1 libegl1 libgbm1`), the floor of any Debian desktop, then runs `ldd` over the extracted AppImage and prints every library still missing before it starts the app, so one run shows the whole list instead of the first missing file.
- **Rule:** a test environment must match what a user has, no less: test an AppImage on the desktop floor, not on a bare image. A smoke test that fails prints everything that is wrong at once. A release dry run (dispatch before the version bump) catches this without a failed release run.

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
