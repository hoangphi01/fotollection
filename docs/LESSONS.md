# Lessons — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-06
>
> What each debugging session taught us, so the same mistake is not made twice. Every bug that took more than one look to find gets an entry here in the same commit as its fix ([DOCS_GUIDE.md](DOCS_GUIDE.md) §2). Newest first. Each entry says what we saw, why it happened, the fix, and the rule to follow from now on. Performance-only lessons stay in [OPTIMIZATION.md](OPTIMIZATION.md) (for example §4.3.1, interactive work off background queues).

## How we debug

1. **Measure or observe before guessing.** The v0.6.0 plan assumed a 32 MP preview might be built; measuring first showed the preview was already capped at 1600 px, and the real gap was zoom ([RESULT.md §11](RESULT.md#11-preview-and-zoom-cost-v060)).
2. **Split the path and log both ends.** A temporary log on the UI side (what was asked) and one on the Rust side (what was answered, how long) tells which layer fails in one try. Name them `DEBUG-…` and remove them before the commit (`grep -rn DEBUG ui/src src-tauri/src` must be empty).
3. **Reproduce with the user's photo when the user sees it.** A synthetic test passed while the real app was soft: unit tests prove the pixels, only the real webview proves what is shown.
4. **When the app hangs, look at every thread before guessing:** `sample <pid> 3` on macOS names what each thread waits for. Rust threads all idle with a spinner on screen means the UI never asked; threads waiting on each other is a deadlock (2026-11-05).
5. **Write the lesson here** before moving on.

## 2026-11-07

### A click on the photo made an undo step named *Crop*, and the white-balance picker never got its click (FR-098, FR-006)

- **Seen:** after using the picker (`W`), History showed *Crop*, the picker stayed armed and the white balance did not change.
- **Why:** the drag that moves a photo in its print crop listens on the stage and captures the pointer on any press; it saved the crop on release even when the pointer had not moved, so a plain click wrote the crop (from none to the centre: a step named *Crop*), and the capture kept the click from reaching the picture, where the picker listens.
- **Fix:** a press becomes a crop drag only after 3 px of movement, and only a drag saves; while the picker is armed the crop drag is off (`use-crop-drag.ts`, `preview-dialog.tsx`).
- **Rule:** a gesture that writes (a drag, a click that picks) starts only past a slop and never on a bare click; two gestures on the same surface must turn each other off, not race for the pointer.

## 2026-11-06

### The v0.13.0 release stopped on its AppStream check: a validator older than the spec we wrote to (FR-099, release)

- **Seen:** the push of `chore(release): v0.13.0` failed CI on `ubuntu-22.04` in *AppStream metadata*: `appstreamcli validate` found `<developer>` unknown and the `vcs-browser` and `contribute` URL types invalid. Nothing was published; the Pages build went out on its own.
- **Why:** the metainfo followed today's AppStream spec (1.0), but the runner's `appstream` is 0.15.2, Ubuntu 22.04's, and it fails on warnings. The file was checked with `xmllint` only, because `appstreamcli` is not on the Mac, so the validator first ran on the release push.
- **Fix:** `<developer_name>` and only the URL types 0.15 knows (homepage, bugtracker); the workflow started again by hand, the version unchanged.
- **Rule:** metainfo uses the tags of the oldest AppStream we support (0.15, Ubuntu 22.04). A file only CI can validate is pushed and checked green before the release commit, never first with it.

### Words in shared rows spilled into their padding: measured by the box, not by the text (FR-099, UI)

- **Seen:** the owner saw *Không RAW* touch its segment's edges; the new `MEASURE=1` overflow check found the same in 15 places across both languages (*Thành phố*, *+ tên nước*, *One colour*, *City, country*, the date formats, *Retro orange*), yet a first version of the check reported only two, and changing the padding seemed to change nothing.
- **Why:** a segment shares its row (`flex-1`, 74 px of an 18 rem panel for three choices) and centres its words; with 10 px of padding a side it had 54 px for text that was 56 to 65 px wide. Centred text spills over both sides, but `scrollWidth` counts only one, so a check built on it saw a 2 px fault where the words had lost 11.
- **Fix:** the check measures the content's own width (a `Range` over it) against the room inside the padding; shared segments keep 4 px a side (`SegmentedToggle`, `SwatchToggle` 6 px beside its swatch), dates set mono a size smaller, and the English place formats say *+ code* and *+ country* as the Vietnamese do.
- **Rule:** an overflow check compares what is drawn (a range's rect) with the room inside the padding, never `scrollWidth` alone; a fixed-width row of words is measured in both languages at the narrowest panel before it is kept.

### One style change took a sweep of twenty files: buttons and rows written by hand (FR-099, UI)

- **Seen:** making remove buttons red, then reset buttons, then the chosen rows of menus and dropdowns meant editing the same classes in about twenty files each time, and some were missed (the compare menu, the dropdown's checked item, the estimate pill); the owner: "too many buttons are fragmented, debugging and refactoring is painful; that is why I said every button must be reused, one class".
- **Why:** each place wrote `<Button variant=… className=…>` with its icon and its chosen state itself, so a meaning (remove, reset, chosen) had no single home.
- **Fix:** `RemoveButton` and `ResetButton` (`components/forms/danger-button.tsx`) and `ListRow` with `CHOSEN_ROW` (`components/forms/list-row.tsx`) hold the look; `check-lego` refuses a hand-made red button, a remove or reset icon in a plain `<Button>`, and `row-chosen` outside the block.
- **Rule:** a button or row with a meaning is a block before its second use, and a check keeps hand-made copies out; a style change is then one edit.

### A card's header grew when a slider moved: a primitive made taller, its containers not measured again (FR-099, UI)

- **Seen:** moving a slider in *Basic* made the card's header jump taller, as its reset button and *edited 1* appeared (owner: "no one adjusts a slider that blinks, this is not a DJ desk").
- **Why:** the header was `min-h-11` with 8 px padding. While icon buttons were 28 px it held them inside 44 px; the same afternoon every button became 32 px (one height for all controls), so a header with a reset needed 48 px, and the reset only shows after the first change.
- **Fix:** the header is a fixed `h-11` without vertical padding; what its action slot holds never resizes it.
- **Rule:** a box that holds something that comes and goes (a reset, a note, a badge) has a fixed size, never `min-h` plus padding; a change to a primitive's size is followed by a look at every container that holds it, in its shown and hidden states.

### Sliders all but vanished in the dark theme: a set-in track checked only in light (FR-099, UI)

- **Seen:** in dark, the sliders' tracks were nearly black on the dark card; only the white thumbs showed where a slider was (owner's screenshot).
- **Why:** the track was `bg-muted` with the `sunken` shadow, tuned on the light board; in dark, `muted` is a few steps from `card` and the inner shadow darkens it further.
- **Fix:** in dark the track is a light brown, the accent mixed into the card.
- **Rule:** every set-in control (track, field, switch) is looked at in both themes beside its card, and is told apart from the card by its colour, not by its shadow alone.

### Remove buttons in four styles: a meaning with no rule of its own (FR-099, UI)

- **Seen:** *Clear all filters* was a plain ghost button, *Clear cache* an outline one, a folder tab's ✕ a faint red one: the owner asked twice for one red style everywhere, Settings included.
- **Why:** each button took the variant of its neighbours, since nothing said that removing has its own look.
- **Fix:** one `destructive` variant (deep red, white ink, burgundy lip) for every button that deletes, clears, removes or stops, and a `check-lego` rule that fails on a ✕ or bin button that is not red.
- **Rule:** a meaning that recurs (remove, pressed, chosen) gets one variant and a check, written when the second case appears, not after the owner finds the third.

### "Like Lightroom's export" built as Lightroom's frame only (FR-093)

- **Seen:** the export dialog had Lightroom's layout, but no subfolder, no file naming, no choice for existing files or metadata; the owner: "I told you to look at Lightroom's export, did you forget?"
- **Why:** the reference was read for its layout; its sections were not listed one by one, so the missing ones were never decided.
- **Fix:** the plan lists every Lightroom section with built, later or never; the missing ones are built in v0.13.0, output sharpening later.
- **Rule:** when the owner says "like X", list every section of X in the plan, each marked built, later or never, before building.

### *New project* dropped every folder without asking: an untitled session lost its unsaved flag on restart (FR-023)

- **Seen:** the owner pressed *New project* on an untitled project with two folders (518 photos, edits on 42) and everything went, with no *Save changes?* question.
- **Why:** the session file records the document block (path and unsaved flag) only for a saved document (`save_project` maps `path` to `DocumentRef`); an untitled one is written with none. At launch, `restore_session` read the missing block as "not edited", so after any restart the untitled work looked saved, and *New* replaced it at once. The dev app restarts often, so the owner hit it the first day.
- **Recovery:** the atomic write keeps the previous session as `session.fotollection.bak`; copying it back over `session.fotollection` with the app closed brought the folders and edits back. Copy the `.bak` aside first: the next autosave overwrites it.
- **Fix:** `document::restored_edited`: an untitled session with photos is unsaved after a restore, a saved document keeps its flag; a test fails on the old rule. *New* also asks for a name first.
- **Rule:** a flag that guards against losing work is derived from what would be lost (an untitled project holding photos), not only from a flag that a restart may drop; anything that replaces the library is tried after a restart, not only in one run.

### The export line pushed the whole panel past the window: `truncate` without a width (FR-093, UI)

- **Seen:** the panel's new one-line export summary (*In ảnh · Một thư mục · chất lượng 95 · Chọn nơi lưu*) ran past the window's right edge, and took the estimate and the *Export* button with it (owner's screenshot).
- **Why:** `truncate` sets `white-space: nowrap` and an ellipsis, but a `<button>` sizes to its content; with no width of its own the long line widened its column instead of being cut.
- **Fix:** `w-full min-w-0` on the line, so it takes the panel's width and the ellipsis shows.
- **Rule:** `truncate` always comes with a width it can be cut to (`w-full min-w-0`, or `min-w-0 flex-1` in a row); a new one-line summary is looked at in Vietnamese with the longest folder name before it is kept.

### Vietnamese marks touched the line above: a new face, the old line heights (FR-099, UI)

- **Seen:** with Be Vietnam Pro, the owner found small Vietnamese text crowded: stacked marks (*ấ*, *ộ*) touched the line above and each other.
- **Why:** labels (`Label`) and dialog titles kept shadcn's `leading-none`, and `text-xs` had Tailwind's 1.33 lines; Geist hid it with its lower marks, Be Vietnam Pro's taller ones need about half a line above the letter.
- **Fix:** `text-xs` and `text-sm` at 1.5 lines in the theme, `leading-snug` for labels and dialog titles, no `leading-none` on text that can hold marks.
- **Rule:** a change of face is checked on Vietnamese at its smallest size, two lines deep, before it is kept (STYLE_RULE §4, "Room for Vietnamese marks").

### The toolbar was cut off in the full-window layout: a column added, the row not measured again (FR-095, UI)

- **Seen:** in the full-window layout the preview's toolbar lost its last buttons past the inspector, and the zoom read *Vừa khung* on two lines, off centre (owner's screenshots, v0.12.0 development).
- **Why:** the left column (18 rem) was added beside the 24 rem inspector, leaving the stage column about 508 px at the 1180 px minimum; the toolbar needs about 660. The row was measured once, in the dialog layout, before the column existed. The zoom showed a word (*Fit*, *Vừa khung*) in a 56 px box.
- **Fix:** v0.13.0 brings the inspector to 18 rem as every panel, shows zoom as a percentage only, moves the count to the filmstrip's label and makes the row a container that drops the zoom buttons when narrow; `MEASURE=1` gains a check for any row or label wider than its box.
- **Rule:** a new column or control re-measures every row it shares, at 1180 px and in Vietnamese, before it is kept; a fixed box never holds a word (STYLE_RULE §4, "Icons first").

### Zoom showed a portrait photo sideways: two ways to the full image, one forgot to turn it (FR-057, v0.11.2)

- **Seen:** the owner zoomed into a portrait GX9 photo in the compare view: both halves showed the sharp part turned a quarter and stretched. The same happens in the single view; it shipped in v0.11.0 and v0.11.1.
- **Why:** the full-resolution image reaches the zoom by two paths. `decode_source` decodes the file and turns it upright (`apply_orientation`) before keeping it for zoom; `full_source`, used when nothing is kept, decoded it and did not turn it. Before v0.11.0 every preview went through `decode_source`, so the second path never ran for the photo on screen. Instant previews (FR-091) open a photo from its standard preview on disk, which keeps no full image, so zoom took the path that had never been right. No test zoomed a turned photo with an empty cache.
- **Fix:** `full_source` turns the decoded image as `decode_source` does. A test zooms a turned photo with nothing kept and fails without the fix.
- **Rule:** when a value can be made in two places, make it in one function; when a new path (a cache, a file on disk) skips the old one, test every consumer with the cache empty, not only after the old path has filled it.

### "Vừa khung" outgrew its segment in the Navigator: words where an icon belonged (FR-095, UI)

- **Seen:** the Navigator's four zoom choices in an 18 rem column: *Vừa khung* ran past both edges of its segment in Vietnamese, while English *Fit* fitted. The same kind of fault as the toolbar's third filter, the label hidden by `@3xl` and the settings buttons too long.
- **Why:** a segment's width is the column's quarter, fixed; the label was chosen in English and never measured in the longer language, and `MEASURE=1` does not yet look for text wider than its box.
- **Fix:** *Fit* and *Fill* are icons (`Scan`, as the zoom buttons', and `Expand`) with their words as the hint; *1:1* and *2:1* stay, short in any language.
- **Rule:** icons first, words as the hint, in any control whose room is fixed (STYLE_RULE §4, "Icons first"); a label that stays is measured in Vietnamese. `MEASURE=1` gains a check for a control whose text is wider than its box (session 7 of v0.12.0).

### Memory state needs one owner, an epoch and one way in: the audit of FR-091 (v0.11.1)

- **Seen:** an audit of v0.11.0's preview memory, read line by line, found no crash but eleven ways it could keep the wrong thing: a develop that ends after *Open* stored its pixels under an id the new library gives another photo; a prefetch became "the newest" and pushed out the photo on screen; every prefetch started a thread of its own, and old ones kept developing RAWs; the background pass and the photo on screen could make the same photo at once into one shared `.part`; the disk limit held only after a whole pass, and then dropped new thumbnails before old previews; a develop with no priority set went first by default.
- **Why:** each cache was written for one caller (the photo on screen) and then reached by three more (the pass, the prefetch, the collage grid), and nothing said who owns an entry, when it stops being valid, or which caller may evict which.
- **Fix:** `clear` bumps an epoch checked under the lock before every insert; `Keep::{No, Behind, Shown}` says where a caller's image goes; one prefetch worker with a queue the latest request replaces; an in-flight set that background work skips (the photo on screen never waits: it runs on a rayon pool); a still-wanted check once a develop's turn comes; previews pruned first, every 100 during the pass; a version in the preview's file name; `Background` unless marked. Each fix has a test that fails without it (checked by removing the fix).
- **Rule:** state kept in memory names its owner, its bound in bytes, what empties it and who may evict whom, in its doc comment, and every cache reachable from more than one caller gets a test per caller. From a pool thread, never wait for work in flight: skip it, or re-check after the wait you already have.

### The measurement caught what the type checker could not (FR-086, UI)

- **Seen:** the Settings window looked right in the dev app, the type check and `check-lego` passed, yet `MEASURE=1` stopped on a page error in its *Memory & cache* section ("maximumFractionDigits value is out of range") and reported a field gap of −2 px in the Pro panel and the photo preview.
- **Why:** `formatMemory` passed `formatNumber` its two digit counts the wrong way round (0, 1): legal types, a range error only when `Intl.NumberFormat` runs. And the slider's wider press area (FR-079, 2026-11-05) was made with `-my-2 py-2`, a negative margin that moved its box over the label above it: nothing moved on screen, but the block owned a margin, which the lego rule forbids.
- **Fix:** the arguments in order; the press area is a `::before` reaching 8 px above and below the track, so the slider's box is what it was.
- **Rule:** run `MEASURE=1` on every UI change before calling it done, in both languages and themes (the `style-audit` skill): it renders every place for real and fails on page errors and on any measure off the table. A wider hit area is a pseudo-element, never a margin.

### A wait inside a pool's job: the preview pass and the best-shot analysis hung each other (FR-091, FR-041)

- **Seen:** the screenshot build's best-shot recording stopped for 20 minutes at 0 % CPU, on the v0.11.0 candidate. `sample` showed a background-pool thread inside the analysis's `par_iter`, which had taken up a photo of the standard-preview pass, asleep in its "wait while a job runs" loop.
- **Why:** `warm_standard_previews` ran its photos with `par_iter` on the background pool and, in each one, slept while any job (an export, an analysis) was registered. The analysis runs on the same pool; a rayon thread waiting for its own parallel work runs other jobs of its pool meanwhile, so the analysis's thread ran a preview photo that waited for the analysis to end, which waited for that thread. The rule of 2026-11-03 and 2026-11-05 again, in a new form: a wait inside a pool's job.
- **Fix:** the pass runs one photo at a time on the plain thread that made the thumbnails, never on a pool; the wait for jobs is outside any pool. Caught before v0.11.0 was published: its release run was cancelled and started again with the fix.
- **Rule:** never block or sleep inside a job of a rayon pool, whatever it waits for; a loop that waits belongs to a plain thread. When the work also needs parallelism, give it a pool no one it waits for uses.

## 2026-11-05

### The dev app was slower than the release for a reason of its own: our crates unoptimized (tooling, FR-091)

- **Seen:** while chasing the spinner, a temporary `DEBUG-hang` log in `protocol.rs` showed each RAW preview taking 1.8 s, one 5.3 s, in the dev app; `measure_preview` gave 0.4 s for the same file in release.
- **Why:** `Cargo.toml` optimizes dependencies in dev (`[profile.dev.package."*"]`) and `fotollection-core` by name, but `"*"` does not cover the workspace's own members: `rawcore`, `rawlgpl` and `oscodec` ran unoptimized, and the RAW develop lives there.
- **Fix:** `opt-level = 3` for the three in the dev profile (DEVELOPMENT §4.1).
- **Rule:** before judging a speed in the dev app, compare with a release measurement; a new workspace crate with hot loops gets its own dev profile line.

### Callers waiting on a pool can starve the work they wait for: the export estimate hung every RAW (FR-016, FR-053)

- **Seen:** in v0.10.0 the owner moved to the next RAW while one was loading, closed the preview and opened a RAW again: the spinner turned for good, and so did every RAW after it. `sample` on the running app showed all eight threads of rayon's global pool in `raw::one_at_a_time`, each waiting for a develop thread, and the develop holding `DEVELOP` waiting for its own `par_iter` work, with the CPU idle.
- **Why:** the preview asks for the export's estimate of the photo shown (FR-053), and the estimate encodes its samples with `par_iter` on the global pool; for a RAW, each sample develops in full. A caller in `one_at_a_time` waits in `join`, which lends its thread to nothing. The develop's own parallel work also went to the global pool, so once every thread of it waited for `DEVELOP` (two or three estimates queued while browsing), the develop holding the lock had no thread left and never ended. The 2026-11-03 fix (a plain thread for the develop) only covered a thread waiting for itself.
- **Fix:** the develop's parallel work runs on a pool of its own (`raw::develop_pool`), used only under `DEVELOP`, so the holder always has its threads whoever waits. Test `develops_asked_from_the_whole_global_pool_all_finish` asks twice as many develops as the global pool has threads from inside it; it hung on the old code (30 s timeout) and passes in 0.01 s.
- **Rule:** work that runs behind a lock gets threads nobody waiting for that lock can hold: its own pool, not the one its callers block. When the app hangs, `sample <pid>` first: it names every thread's wait in one look.

### The spinner that waited for a load nobody watched (FR-057, FR-016)

- **Seen:** with the develop's hang fixed, the owner opened a RAW again and its spinner turned for good. `sample` showed every Rust thread idle and no request in flight: Rust had answered all it was asked, so the stage never asked for this photo.
- **Why:** `useShownSrc` loads one render at a time, so a slider's drag cannot start a request per frame. It also made **another photo** wait for the old photo's load, and it held that load only through the `Image`'s own handlers. A RAW still developing kept the next photo waiting; and an `Image` nobody references can be collected by WebKit before its `load` fires (the likeliest cause; it was not caught in the act), after which `loading` stayed set for good, so every photo after it waited in that dialog, which stays mounted while closed.
- **Fix:** the loading image is kept in a ref; another photo lets the old load go and starts at once (only renders of the same photo wait for each other); a closed dialog lets go of its load. Not covered by a test (the UI has no unit tests yet): checked by the owner in the dev app.
- **Rule:** when the UI waits for Rust, `sample` the app first: idle threads with a spinner mean the request was never sent. Keep a reference to every `Image` you wait on, and never make one photo's load wait for another's.

### Two locks taken in both orders: found by a sweep, not yet seen hanging (FR-031, FR-023)

- **Seen:** after the hang above, the owner asked whether others like it were left. A script listed every function taking two of `AppState`'s locks and the order it takes them in; each pair taken in both orders was read to see whether both guards are held at once.
- **Why:** `analyze_library` holds `analyses` while it reads `library`, and `borrow_and_apply` held a read of `library` while it took `analyses`. `install_settings` held `frame` for writing while it read `settings` and `ui`, and `save_settings` holds reads of all three, `settings` first. Two reads of an `RwLock` share it, but std's `RwLock` lets no new reader in while a writer waits, so a third thread writing (a rating, a slider) closes the circle and the three wait for good.
- **Fix:** `borrow_and_apply` lets the library go before it takes `analyses`; `install_settings` reads `settings` and `ui` before it takes `frame`. Every other pair that is held at once is taken in one order: `document_lock` → `library` → `pending` → `document`; `library` → `analyses`; `settings` → `frame` → `ui`; `running` → `busy` / `live` in the camera.
- **Rule:** hold one lock at a time where you can (read what you need, let it go, take the next); where two must be held together, take them in the order above. A read lock is no exception: a waiting writer turns it into a wait.

### A slider's knob reaches 7 px past its track: a 4 px inset clipped it (UI)

- **Seen:** the owner found the last slider of Faves' *More* section cut at the bottom while hovered, just above the *Less* button.
- **Why:** `Collapse` clips its content to animate its height, and kept 4 px around it for focus rings. A slider's 12 px knob is absolutely placed on a 4 px track, so it adds no height to the row; with its 3 px hover ring it reaches 7 px below the track.
- **Fix:** the inset is 8 px (`-m-2 p-2`, which cancel, so nothing moves).
- **Rule:** a container that clips leaves room for what its blocks draw outside their own box (rings, knobs, shadows); check the last block of every clipping container with its hover and focus states on.

### A score on block averages chose the wrong camera fit (FR-072)

- **Seen:** "Like the camera's JPEG" picked three curves for the R10 and the A7 III, which then sat further from the camera's JPEG (4.76 and 4.44 against 4.42 and 3.29 in ΔE2000).
- **Why:** the two fits were scored on 48 × 32 block averages with CIE76: a fit can match every block's mean and still miss the colours inside it, and CIE76 weighs saturated colours too much.
- **Fix:** score pixel by pixel on a 480 px copy, ΔE2000 against the JPEG averaged down to it; all seven Bayer samples now pick the fit the full-size measurement prefers.
- **Rule:** a selector must agree with the measurement it stands in for: check its picks against the full measurement on every sample before trusting it.

### A build beside the editor ran the Mac out of memory (tooling, 2026-11-05)

- **Seen:** a release build of the examples, then a measurement of 24 MP RAW files, ended with the process killed (exit 137) and the machine restarting; the scratch folder in `/tmp` went with it, the RAW samples included.
- **Why:** the editor's own tools held about 5 GB (rust-analyzer 1.9 GB, the C/C++ extension 1 GB, VS Code's helpers), the swap was nearly full, and cargo built on every core while a develop held a 24 MP image several times over.
- **Rule:** when `scripts/resources.sh` shows little free memory, build with `CARGO_BUILD_JOBS=4`, never run the dev app beside a build or a test, measure large files one at a time, and keep anything worth keeping outside `/tmp`.

### A label that hides leaves its icon off centre; a size read from a whole file (FR-035, UI)

- **Seen:** the owner found the toolbar's Auto turn, Revert and Select icons a little off centre, and Auto turn 3–5 s late on 518 photos.
- **Why:** each button set its icon as `data-icon="inline-start"`, which gives the button 6 px of padding on the left and 10 on the right for a label; below the `@3xl` width the label hides and the icon sits 2 px left of centre. And Auto turn asked every photo's size by reading the whole file, 4.4 GB for the trip's folder, one file after another: 3.7 s.
- **Fix:** `ToolbarButton` (components/forms): a square button with the icon centred when the label hides, the label's padding only when it shows, a spinner while its work runs; `check-lego` refuses a hand-hidden label, and `MEASURE=1` measures every icon-only button (it failed on the old code at 2 px). The size reads the first megabyte (0.4 s for the folder).
- **Rule:** an icon-only state of a button is a layout of its own: measure it (`MEASURE=1`), do not trust padding written for the labelled one. Before reading a file to learn one fact, check how much of the file that fact needs.

### Two readers make up the end of a RAW file cut short (FR-016, FR-072)

- **Seen:** testing damaged copies of the owner's GX9 files, a RW2 cut to 10 % previewed and exported "fine": macOS's ImageIO filled the missing 90 % with coloured stripes. Then a CR2 cut to 20 % developed through our own path with diagonal stripes: rawler's CR2 reader reads on past the end without an error.
- **Why:** both readers treat missing data as data. On a Mac, Panasonic and Olympus go to ImageIO first (for its lens correction), and any format reaches it when our develop fails; since session 16, CR2 also goes through rawler's reader first, where before it went to ImageIO, which refuses that file.
- **Fix:** `rawcore::cut_short` (every strip and tile must end inside the file) before any develop, and `raw::intact` (rawler or our DNG reader reads the sensor data through) before the system's codec; a test cuts each CC0 sample to 40 % and expects the chain to refuse it, and failed on the old code for the CR2.
- **Rule:** a decoder's "ok" on a damaged file is not trusted: make damaged copies of a good file (cut at several points, inside and past its image data, flipped, overwritten) and look at what comes out, through every reader the chain can fall back to. Where the image data ends depends on the format: a CR2's ends at half the file, so a test that cuts at 60 % proves nothing.

### A method's name is not a result: RCD's own step scored worse here (FR-072)

- **Seen:** the plan said "RCD"; written from its description, its ratio-corrected green was the weakest of three variants on the zone plate (15.3 dB against Malvar's 17.3) and second on the Siemens star.
- **Why:** a published method is tuned for its authors' pipeline and test images; on ours (balanced before demosaicing, these charts), a Laplacian-corrected estimate per side, weighted by gradient, did better (star 37.9 dB, colour edges 10.4).
- **Fix:** the variant the charts kept, with RCD's direction decision, named `directional` and described as what it is, in the code and the docs.
- **Rule:** score every variant of a step on the charts before naming it in a plan, a doc or the website; ship the measured one under an honest name, and state any score that got worse.

### A score against the camera's JPEG mixes the develop and the fit (FR-072)

- **Seen:** once our develop took every Bayer format, three samples were further from the camera's JPEG than rawler's develop (D750 2.26 against 1.18), though the S5's greys were more neutral.
- **Why:** each path fits its own tone to the embedded JPEG: rawler's with three curves, ours with one curve and a matrix. The score measured both steps at once.
- **Fix:** none needed in the develop: finished with rawler's fit, ours scored at least as well on every sample tried. The fit is chosen per file in session 17.
- **Rule:** when a pipeline gets worse against a reference, swap one stage at a time between the old and the new path before changing either. The samples folder can hold folders too: a test that walks it skips anything that is not a file.

### A feature built for one photo must be checked in Collage mode (owner, 2026-11-05)

- **Seen:** in Collage mode the colour sliders seemed to change only the second photo of a sheet, and they stayed on screen with no photo picked.
- **Why:** the Collage panel and the sheet's preview edited the shared look, built for the gallery; a photo with a look of its own (set in its preview) ignores the shared one, so only the other photo moved. The panel had no "no photo picked" state at all.
- **Rule:** every change from now on is checked against Collage mode too: two photos to a sheet, each with its own edits; what the picked cell means; what shows when none is picked; the sheet's preview as well as the panel; postcards. DOCS_GUIDE's checklist names it.

### A code commit without docs slipped through the local check (docs)

- **Seen:** after the v0.9.9 push, the `docs-with-code` workflow failed on 9b0d846 (the badges' centring), which changed two UI files and no doc; it was not the last commit, and the release itself was not blocked.
- **Why:** `scripts/check-docs.sh` runs `check-docs-with-code.sh` on the working tree or the last commit only; CI checks every pushed commit.
- **Rule:** before a push, run `scripts/check-docs-with-code.sh origin/main..HEAD`, as CI does; a fix too small for a doc row still gets its line in the CHANGELOG, or `[no-docs]` with a reason in its message.

### A third filter did not fit the toolbar (UI)

- **Seen:** a third dropdown (RAW or not) was hidden under the right-hand panel at the minimum window, 141–186 px short; nothing reported it, and `check-lego` missed the toolbar's small selects because their props span several lines.
- **Why:** the toolbar has a floor (STYLE_RULE §4), and two dropdowns already filled it.
- **Rule:** measure a new toolbar control at `minWidth` in both languages before keeping it; group filters behind one **Filters** button. `check-lego` reads one line at a time: a rule about a JSX prop must allow for props written on several lines.

### Capitals in a badge sit high (UI)

- **Seen:** the RAW and JPEG badges looked a little high; the first fix (`pt-px`) moved the letters higher still.
- **Why:** flexbox centres the line box, and capitals have no descenders, so the letters sit in its upper part. `pt-px` lowered the top padding from 2 px to 1 px, which lifts the line.
- **Rule:** centre capitals on the letters: a one-line-high box (`leading-none`) lowered 1 px (`pt-px pb-0`). Check by counting the pixels between the letters and the border on a zoomed screenshot (here 38 / 51 px at 8× before, 46 / 43 after), never by eye alone.

### Top-level keys of settings.json are snake_case (tests)

- **Seen:** a v13 fixture written with `"recentAddresses"` would not load ("unknown field").
- **Why:** inside `render`, `frame` and `ui` the keys are camelCase, but the file's own top-level keys (`schema_version`, `recent_addresses`) are snake_case, and unknown fields are refused.
- **Rule:** write a fixture by saving one from the code, or copy the previous fixture; never type a key from the docs' example.

### A preview that restarts on every step never lands during a drag (FR-079)

- **Seen:** dragging a colour slider, the preview hardly moved, then jumped when the drag ended; the owner compared it with Lightroom.
- **Why:** three costs on every step. The webview started a new image for each new URL and dropped the one still loading, so with a step every 16 ms and a render of 20–40 ms none finished; Rust resized the photo again although only the look changed; and the main thread wrote `settings.json` with an fsync.
- **Rule:** a live view loads one image at a time and the newest next (`use-shown-src.ts`); what a gesture does not change is kept between its steps (`PreviewCache::resized`); a file written for a setting is written off the main thread, coalesced, and flushed on quit. Measure one step of the gesture (`measure_preview LOOK_DRAG=1`) before and after.

## 2026-11-04

### Two kinds of tooltip, and only one knew about the Windows title bar (FR-052)

- **Seen:** on Windows, the tooltip of *Crop* in the preview ran under the app's own title bar; on macOS it looked right.
- **Why:** STYLE_RULE asks for one hint layer (`data-hint`), but four places still used shadcn's `Tooltip`, which opens above its button and knows nothing of the 32 px title bar the app draws on Windows.
- **Fix:** all four moved to `data-hint`, which opens below.
- **Rule:** one kind of tooltip in the app. When a rule names the shared piece, search for the other ones it replaced (`grep -rn "<Tooltip" ui/src`) before calling the rule kept.

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

### A tooltip placed by a guessed height runs off the window (FR-057)

- **Seen:** the zoom buttons' three-line hint, at the bottom of the preview, was cut off by the window's edge.
- **Why:** `HintLayer` chose above or below with a fixed 40 px allowance, the height of a one-line hint; the new hint was three lines.
- **Rule:** place a floating element after measuring it (`useLayoutEffect`, hidden until placed), never by an assumed size. Check every hint near a window edge with its longest text, in both languages.

### The demo recording exits at once while the dev app runs (screenshots)

- **Seen:** `FOTOLLECTION_DEMO_FOLDER=… target/debug/fotollection` returned in 0.1 s and wrote no dump.
- **Why:** the single-instance plugin handed the launch to the `tauri dev` app already running, and quit.
- **Rule:** stop the dev app before recording a demo dump (`site/screenshots/build.sh`), then start it again.

### A folded Collapse still takes its stack's gap (UI)

- **Seen:** with *3 mm border* off, its hint sat 12 px under the switch instead of 6; a postcard's cells card had a 24 px gap where its cut-line option was folded.
- **Why:** a folded `Collapse` is 0 px tall but is still a child of a `space-y` stack, so the gaps on both of its sides add up.
- **Rule:** every stack block sets `--stack-gap`, and a folded `Collapse` pulls itself up by it (`collapse.tsx`). Measure with every switch both on and off.

### A style rule without numbers and a check lets the drift back (UI)

- **Seen:** after one card's spacing was fixed, the same kind of fault was still in other cards: six values for the space inside a field, five label styles, selects of 28 and 32 px side by side, a preview spaced 20 px against the panel's 12 px, and a right panel that looked narrower than the folder list though both were 288 px.
- **Why:** STYLE_RULE said "use the shared pieces" but gave no measures, and nothing failed when a card spaced itself by hand. The right panel's native scroll bar and its extra padding took width the folder list's overlay bar does not.
- **Rule:** the measures are a table in STYLE_RULE §4 (the lego rule), each owned by one block, and `check-lego.mjs` in `npm run lint` fails on hand-made spacing, sizes and toggle groups in `src/features`. Prove a panel change with a measurement table across every mode (`getBoundingClientRect`), every cell the same value.

### A lazy image with no height moves the anchors below it (website)

- **Seen:** the navbar's Coffee button scrolled to a point 800 to 1,800 px above the support section; every other anchor landed right.
- **Why:** the benchmark charts above it were `loading="lazy"` with a width and no height. The page scrolled toward `#support` with each chart counted at zero height; as they loaded on the way, the section moved down and the scroll stopped short.
- **Rule:** every lazy image on the site carries both `width` and `height` (charts get theirs from `site/_data/charts.json`, written by `bench/chart.mjs`). After a layout change, click every in-page link and check where it lands, not only that it scrolls.

### A fit check must count only what prints (FR-073)

- **Seen:** a postcard address that printed whole on its four lines still showed the red "does not fit" mark.
- **Why:** the text ended with a line break, and `wrap` turned it into one more, empty row, which printed nothing but counted as a fifth line. Rows past the last line were also measured with no room, so each left-over letter became a row of its own.
- **Rule:** drop trailing blank rows before counting, measure rows past the last line as long as it, and test the check on text as the app's field sends it (doubled spaces, a trailing break), not only on tidy strings.

### Faux bold by drawing twice doubles a thin face (FR-073)

- **Seen:** the owner's postcard showed bold "Minh" with a doubled M, two thin strokes side by side.
- **Why:** bold was the text drawn a second time 0.045 em to the right. Xanh Mono's strokes are thinner than that offset at print size, so the two copies did not touch.
- **Rule:** to embolden, draw at steps of at most half a pixel across the extra width and keep each pixel's highest coverage, then blend once (`text::draw_text_bold`); a test checks one run per stem. Look at a print-size render of a thin face before trusting a style.

### Screenshot steps that pick photos by position (screenshots)

- **Seen:** after sixteen Prague photos joined the demo set, `shoot.mjs` timed out waiting for the crop photo's tile, then for a fifth photo in a folder tab.
- **Why:** the basket and set-aside photos were chosen by their position in the list (`TRIP_QUEUES`), so a new set put the crop photo in *Set aside*, out of the gallery; the folder tabs were matched by name and kept fewer photos than the clicks assumed. The gallery is virtualized too, so a tile far down the list is not in the page until it is scrolled to.
- **Rule:** when the demo set changes, check that every position still names a photo the shot does not show, and reach a photo's tile with `reach()` (it scrolls the grid) instead of assuming it is drawn. Screenshot the page when a wait times out; the cause is on screen.

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

## 2026-10-30

### A comment in a heading changes its anchor (manual, ADR-0031)

- **Seen:** the docs workflow failed after the manual's chapter anchors went into the guide's titles (`## 1. Choosing the photos to print <!-- choose -->`): links from PRD and STATUS to `USER_GUIDE.md#1-choosing-the-photos-to-print` could no longer be found.
- **Why:** lychee builds a heading's anchor from its raw text, comment included, although GitHub does not show the comment. `check-docs.sh` does not check anchors, so only CI saw it.
- **Rule:** keep markers out of headings: put them on their own line above (`<!-- chapter: choose -->`). After renaming or marking a heading, `grep` the docs for links to its old anchor before pushing.

### A narrow column left on its own looks broken (manual)

- **Seen:** the owner found the manual "pushed to one side": its text had `max-width: 880px` inside the 1180 px `.wrap`, left-aligned, so a wide empty band ran down the right.
- **Why:** reading width was chosen for the text alone, without comparing the page with the landing page, whose sections fill the `.wrap`.
- **Rule:** on the website, content spans the `.wrap` like the landing page's sections ([STYLE_RULE.md §4](STYLE_RULE.md#website-site)); measure that the left and right margins match before pushing.

### A screenshot of the start-up screen (website screenshots)

- **Seen:** the manual's frame-card shot came out as the logo and a loading bar, and a full shoot took almost 8 minutes instead of under 2.
- **Why:** the mock IPC bridge in `shoot.mjs` did not answer `startup_finished`, so the start-up screen (FR-060) stayed until its 15-second safety timer; shots taken right after loading caught it. `scrollIntoView` was not the cause, though it also scrolls the page, not only the panel.
- **Rule:** when the app gains a command it calls at start-up, answer it in the mock bridge; `open()` now waits until `#splash` is gone, and a shot that needs a scrolled panel scrolls that panel's own box.

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

### Content from a document takes the app's layout, not the document's (FR-110)

- **Seen:** the first manual inside the app showed each chapter as one long page of prose on the window's ground, as the website does, and the chapter list ran past its column's edge (owner, 2026-11-07: "style broken; every context follows the style rule").
- **Why:** the Markdown was rendered as a page, with its own spacing and widths, beside an app built from cards; and Radix's `ScrollArea` lays its viewport's content out as a table, which grows to its widest row, so `truncate` never took effect in the list.
- **Fix:** `cardsOf` cuts a chapter into the app's own `SectionCard`s (its opening, each section, each bold-led paragraph and step) in the Settings window's 36 rem column; keys show as `Kbd`, captions as `Hint`. The list's viewport is `block`, as the cards' already was.
- **Rule:** text or pictures brought into the app from a document (the guide, the CHANGELOG) give only their words; the layout is the app's blocks (STYLE_RULE §4). A list in a `ScrollArea` that must truncate sets its viewport's child to `block`.

### A shadcn primitive the app never used brings its own look (FR-110)

- **Seen:** the manual's search was an `InputGroup` with an icon; it showed two boxes, the group's frame and the set-in `Input` inside it, with a seam after the icon (owner, 2026-11-07: "the text field is broken; see how the other fields do it").
- **Why:** every text field in the app is a plain `Input`, which carries the set-in depth (FR-099). `InputGroup` came from shadcn untouched, so its own frame and the inner field's both drew.
- **Fix:** a plain `Input`, as every other field; `check-lego` now refuses `InputGroup` in the features.
- **Rule:** before a new kind of control, find how the app already does that job (grep the features) and use the same block; a primitive no feature uses yet is unstyled for this app until it is checked in both themes.

### A check that is printed is not a check that gates (docs)

- **Seen:** `check-docs.sh` reported a Markdown problem, and the commit chained after it still went in (`scripts/check-docs.sh | tail -1; git commit …`).
- **Why:** with `;` (and through a pipe), the next command runs whatever the check returned; reading the last line afterwards is too late.
- **Fix:** the extra blank line fixed in the next commit.
- **Rule:** a commit follows its checks with `&&`, never `;` or a pipe: `scripts/check-docs.sh && git commit …` (DOCS_GUIDE §3).

### Two lists drew their rows with two measures (STYLE_RULE §4)

- **Seen:** the folder list put 8 px between a row's icon and its words, Settings' sections 4 px, with a 14 px icon and smaller words (owner, 2026-11-07: "the room between icon and words is not the same").
- **Why:** the folder list's rows are its own (they drag), and `ListRow` took its gap, icon and text size from the small `Button`; the table in §4 fixed only the room between rows.
- **Fix:** one measure, `ROW_LAYOUT` and `ROW_ICON` in `list-row.tsx`, taken by `ListRow` and by the folder list's rows; the §4 table now gives the row's inside as well.
- **Rule:** a block written twice shares its classes from one constant, and the §4 table states every measure a reader would compare, inside the row as well as between rows.

### A list shown only while a field is empty vanishes after its first use (FR-077)

- **Seen:** on a postcard, *Recent addresses* showed once: a click put the address on the card, and the list was gone; on a card that had an address it never showed. Under the address, the two link-styled choices ("Use for 515 postcards without an address", "or for all 517 other postcards…") were cut at the panel's edge (owner, 2026-11-07).
- **Why:** the list was drawn only while the address was empty, so picking from it hid it; the choices were link buttons, which never wrap, and `check-lego` still let this one file use them.
- **Fix:** the addresses are listed on every card as `ListRow`s, the card's own marked; the choices are a `PanelNote` with two buttons, *Use for them* and *Use for all N*, its words wrapping in the panel; the file's `check-lego` exception is gone.
- **Rule:** a list to pick from stays while its pick is shown, marking it, so the user can change it. An exception in `check-lego` is closed the next time its file changes, as its reason promised.

### A field that saves while it is typed hands its drafts to everything that listens (FR-077)

- **Seen:** *Recent addresses* held one address five times, each cut at a different place (owner, 2026-11-07).
- **Why:** the address field saves 600 ms after the last key, so the preview follows the text, and the save also remembered the address: every pause in typing added a draft.
- **Fix:** saving no longer remembers; the field calls `remember_postcard_address` when it is left (`CommitInput onDone`), and the start of an address already kept gives way to the whole one, also in a list read from the file (`fr_077_a_draft_of_an_address_gives_way_to_the_whole_one`).
- **Rule:** a live save is a draft. Anything that keeps a history of what was typed (recent items, an undo step's name, a log) listens for the end of typing, not for the save.

### An automatic check covers only what it reads (STYLE_RULE §4)

- **Seen:** a style audit of the whole app (2026-11-07, owner: "STYLE_RULE is the source of truth, all three modes") found about 97 faults `check-lego` passed: a hand-made toggle group in the gallery's toolbar, `accent-white`, `rgb()` shadows, 24 px steppers, a hard-coded `htmlFor`, zoom buttons made `ghost` by an expression, 28 px toolbar buttons, hints that never showed on disabled buttons, cards that grew on the first slider step.
- **Why:** it read only `src/features`, so `app.tsx` and the blocks went unchecked; whole files were exempt for reasons that no longer held (a file gone, a switch no longer in the title bar); its patterns matched literal text only (`variant="ghost"`, not `variant={on ? 'ghost' : …}`), and one used `\b` before `rgb(`, which never matches after `_` in a Tailwind class (`shadow-[…_rgb(…)]`): the probe proved it before it was trusted.
- **Fix:** the shared blocks fixed once (ToolbarButton 32 × 32 with its hint on a wrapper, `text-xs` and 16 px icons in small buttons, `ROW_HOVER`, `StageNav`, `SectionCard onToggle`, `ReportButton`, `HistoryButtons`, `outside-dim`), each mode moved onto them, and `check-lego` reads `app.tsx` and the blocks, drops the stale exemptions and refuses hand-written colours, icon buttons under 32 px, hard-coded `htmlFor` and ghost or link by expression; each new rule shown to fail on a probe file first.
- **Rule:** a check is listed with what it reads. When a fault passes it, widen what it reads before adding a pattern, and prove every pattern fails on the old code.

### A row added above a stage takes its height from whatever sized the picture (FR-035)

- **Seen:** after the sheet preview's tools moved into a toolbar row above the stage, a square 10 × 10 sheet ran past the stage's foot: its bottom and the chosen photo's outline were cut (owner, 2026-11-07: "you did not check the bug when choosing a size").
- **Why:** the sheet's image is capped by a height written for the dialog without a toolbar (`90vh − 4rem`, the stage's padding only); the new 64 px row and its line came out of the stage, not out of that number. A landscape sheet still fit, so only a tall or square one showed it.
- **Fix:** the cap takes the toolbar row too (`calc(90vh − 4rem − var(--spacing-header) − 1px)`).
- **Rule:** after changing what surrounds a stage (a toolbar, a header, a strip), open it with every shape it shows (landscape, portrait, square, a postcard) and check the picture's four edges; a size written as a calc names every part it subtracts.

### A flex child that holds a sentence needs `min-w-0` (FR-043)

- **Seen:** with a long custom print size chosen ("Demo · 100 × 100 cm · 1:1"), the red delete button beside the size was pushed 27 px out of the "Khổ tấm in" card (owner, 2026-11-07).
- **Why:** shadcn's `SelectTrigger` is `whitespace-nowrap` with no `min-w-0`: a flex item's minimum width is its content, so the trigger would not shrink, and its `line-clamp` never cut anything. The demo recording had no custom size, so no check ever drew that row.
- **Fix:** `SelectTrigger` and its value are `min-w-0`, the chosen custom size's name is the trigger's hint, and a stress fixture (a long custom size chosen) opens the row in every run; `UI-LAYOUT-001` caught it before the fix.
- **Rule:** an element that holds words in a flex row gets `min-w-0`, or it pushes its neighbours out. A state the recording lacks gets a fixture, or no check sees it.

### A checker's own faults look like the app's (UI_TESTS)

- **Seen:** the first run of the UI tests reported 1,361 overlaps, a WebKit timeout in the manual, 11.9 px gaps and a pixel difference of 7 % between two runs of the same build.
- **Why:** overlaps counted controls under an open dialog or scrolled out of their panel; `settle` waited for lazy pictures WebKit never starts; WebKit lays out in fractions of a pixel; the panel's scroll position differed between runs.
- **Fix:** a control counts only where a click reaches it (on top at its centre, inside every box that clips it), within one layer; lazy pictures out of view are not waited for; measures are whole pixels; every scrolling box is at its top before a picture is taken; and an approved set is compared with a second run before it is trusted.
- **Rule:** a new detector is run twice on an unchanged build and every report of it is read before it is believed; a check that cannot fail proves nothing, and one that fails on nothing is as useless.

### A check that runs only in CI is met only at release (SECURITY.md §7)

- **Seen:** v0.13.2's release stopped in CI's security job: "dangerous DOM API found" in `manual-text.tsx`, a comment that said the manual uses no `innerHTML`.
- **Why:** the job greps the words themselves in every `.ts`/`.tsx` of `ui/src`, comments included, and nothing local ran the same grep, so the first run of it was the release.
- **Fix:** the comment says "no HTML from text"; `check-lego` (`npm run lint`) runs CI's grep on every file before each commit.
- **Rule:** every check CI runs on the code also runs in a local command before a push; when one fails only in CI, its twin goes into the local checks in the fix's commit.

### A table in linear light is coarse where the eye is fine (FR-088)

- **Seen:** RAW shadows had bands of three or four 8-bit levels (RESULT §28's "gaps"), lifted or not.
- **Why:** the tone curve's 4,096 bins are even in linear light over 0–4, so one bin near black spans about three output levels, and each pixel read its bin alone.
- **Fix:** read the curve between its two nearest bins (`Curve::at`); a dark ramp in a test failed with steps of 4 on the old code first.
- **Rule:** a lookup table indexed in linear light is interpolated, or indexed in an encoded domain; work out how many output levels one bin spans near black before trusting it, and measure the gaps before and after.

### `select-none` on the app's root left every dialog selectable (FR-099)

- **Seen:** nearly any text could be highlighted, a card's title in the photo preview among them, and a right click opened the web view's menu with *Inspect Element* (owner, 2026-11-08: "this is an app, not a web page").
- **Why:** `#root` was `select-none`, but Radix draws dialogs, popovers and menus into portals under `<body>`, outside `#root`, so the preview, Settings and every menu were selectable; nothing refused the web view's context menu or keys. *Inspect Element* itself was the dev build's (releases have no devtools).
- **Fix:** selection off on `html` and `body`, back on for fields and `Selectable` (a photo's facts, error messages); `lib/no-web.ts` refuses the context menu, the browser keys and image drags; `check-lego` refuses selectable text by hand, `check-docs` refuses devtools in releases.
- **Rule:** a rule meant for the whole app is set on `html`/`body`, never on `#root`: portals live outside it. Check a new global rule inside a dialog and a popover, not only on the main view.

### A content security policy breaks only the release build (SECURITY.md §2)

- **Seen:** a security review (2026-11-08) found `connect-src` without the app's own `fotothumb:` protocol, while the interface `fetch()`es looks' tables and errors from it; the GPU look and the hover preview would fall back or fail in releases, silently, and every dev session looked fine.
- **Why:** the dev app loads the page from Vite and applies no CSP; only the built app does, so a CSP gap never shows while developing.
- **Fix:** `connect-src` names `fotothumb:` and `http://fotothumb.localhost`; a test of the CSP and a release-build check before each release follow (the security checks).
- **Rule:** anything the page loads or fetches is checked against the CSP when written, and the release build is opened before a release with an eye on CSP violations.

### The histogram was soft on Windows only: a stretched, blended SVG is scaled as a picture (FR-098)

- **Seen:** the histogram's curves looked blurred on Windows, sharp on a Mac (owner, 2026-11-08).
- **Why:** the SVG was a 255 × 1000 `viewBox` stretched with `preserveAspectRatio="none"`, its channels blended with `mix-blend-mode`. Chromium (WebView2) draws a blended layer once as a bitmap and scales it; at Windows' 125 % or 150 % that scaling interpolates, while a Retina Mac at exactly 2× hides it.
- **Fix:** `HistogramPlot` draws on a canvas sized in device pixels (`clientWidth × devicePixelRatio`), blending with the canvas's own `multiply`/`screen`, redrawn on resize, on a new display scale and on a theme change.
- **Rule:** a picture drawn from numbers (curves, charts) is drawn at the device's pixels, never a stretched SVG with blend modes; check it at a fractional scale (125 %, 150 %), not only on a Retina screen.

### The histogram drew a straight slope: a `moveTo` started a new shape (FR-098)

- **Seen:** on a night photo the blue channel was a straight line from its peak down to the right end (owner, 2026-11-08).
- **Why:** the canvas traced each channel with `moveTo` at its first level after the area's foot `moveTo(0, h)`; the second `moveTo` began a new shape, which `closePath` closed back to the first level, a full one on a night photo. Rust's counts were right.
- **Fix:** an area's points are all `lineTo` after its foot; only the light's line starts with `moveTo` (`histogram-plot.tsx`).
- **Rule:** when a picture of numbers looks odd, check the numbers first, then the drawing; in a canvas path, one `moveTo` per shape.

### The inspector's foot flashed on each next photo: a hint read `current`, not the last answer (FR-095)

- **Seen:** pressing → in the photo preview, the key hints at the foot of the right column jumped once (owner, 2026-11-08).
- **Why:** `canDrag` read `useLiveQuery(…).current`, which is `null` from the moment the photo changes until Rust answers; the *Drag to frame* hint left and came back, and the centred lines moved. A postcard's reuse note read `current` the same way and blinked as cards were flipped.
- **Fix:** what is only shown (the hint, the cursor, the note) reads `.data`, the last answer; what acts on the photo (the drag, the crop editor) still waits for `.current`.
- **Rule:** in `useLiveQuery`, `.data` is for what the user sees, `.current` only for what changes the photo.

### A list that should scroll spilled out of its box: `max-h` on a ScrollArea never reached its viewport

- **Seen:** a scrolling list drew its last rows over what followed it (owner, 2026-11-08, on v0.14's look list; v0.13's place-borrowing and clock dialogs had the same shape).
- **Why:** shadcn's `ScrollArea` puts the class on its root and gives the viewport `size-full`, 100 % of the root's height. A root with only `max-h-*` has an auto height, so the viewport grew with its rows, never scrolled, and spilled out of the root, which does not clip.
- **Fix:** the root is a column and the viewport `min-h-0 flex-1` (`components/ui/scroll-area.tsx`), so a `max-h` on the root bounds the viewport.
- **Rule:** a height limit is checked where the scrolling happens, not where the class is written.
