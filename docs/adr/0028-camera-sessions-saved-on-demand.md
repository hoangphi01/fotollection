# ADR-0028: Camera sessions in a temporary folder, saved on demand

- **Status:** Accepted
- **Date:** 2026-10-26
- **Related:** FR-062, FR-063, FR-056, FR-023, NFR-006, NFR-018, NFR-019, ADR-0026, ADR-0024

## Context

The Sony driver works from Fotollection's own code on Windows: a shot is in memory 1.1–1.7 s after the press, and live view reaches 30 fps ([RESULT §13.1](../RESULT.md#131-sony-a6400-from-fotollections-own-code-windows)). FR-063 as first written saved every shot straight to `Pictures/Fotollection/Camera/<date>/`. The owner wants a different flow, closer to how a print shop or an event works:

1. **Connect camera** looks for the body.
2. A capture screen opens. It is laid out like the photo preview, with camera controls and a column of the shots just taken where the preview has its inspector.
3. Shots land in a temporary folder that shows as a new tab.
4. Closing the capture screen shows that tab's gallery, which offers **Save all**. Only then does the user pick where the photos go.

Most shots of a session are tests or rejects, so the user should not have to choose a folder before shooting, or tidy one afterwards.

Two constraints also apply:

- **macOS:** the macOS transport (ImageCaptureCore) must run on the main thread, which in the app is Tauri's event loop.
- **Shot detection on Windows:** through WPD the a6400's "new photo" event does not arrive. A shot is noticed from the shot state in the settings block ([cameras/sony.md](../cameras/sony.md)).

## Options considered

1. **Save straight to a fixed Pictures folder** (FR-063 as first written).
   - For: nothing to decide afterwards.
   - Against: every test shot ends up in the user's library, and the user has no say in where the folder goes.
2. **Ask for a folder before shooting.**
   - For: simple.
   - Against: an extra step before the first shot, which is the moment that matters in a booth.
3. **A temporary session folder in the app's data directory, shown as a tab, with Save all and Discard.**
   - For: shooting starts at once. The usual tools work on the shots (crop, stamp, cull, export). The user chooses the destination only for what they keep.
   - Against: files live in the app's data directory until saved, so closing the tab must ask, and a forgotten session stays on disk.
4. **Keep shots in memory only.**
   - Against: RAW files are 25 MB each, which breaks NFR-004 on a long session. A crash would lose the shoot.

## Decision

**Option 3.**

**Where shots go.** Each connection gets a folder `camera/<YYYY-MM-DD HHmmss>/` under the app's data directory, not the system temp folder, which the OS may empty. Shots are written atomically, as a `.part` file renamed when complete. The folder is added as an ordinary tab, without subfolders.

**How a session tab is recognised.** It is recognised by its path, being under the app's `camera/` directory. The project format does not change.

**Save all.**

1. The user picks a destination with a Rust-side dialog.
2. Each file moves there: a rename on the same volume. Across volumes, the file is copied, its size is checked, and only then is the original removed.
3. An existing file is never overwritten: a name already taken gets a suffix.
4. The tab is then pointed at the destination through the same path as Relink (`project::restore_into`), so crops, stamps and ratings stay.

**Discard.** Closing a session tab that still has unsaved shots asks first, with **Save all**, **Delete photos** or **Cancel**. Delete removes only the files the session created.

**Live view.** Frames stay in memory and reach the UI through the `fotothumb` protocol. They are never written to disk or sent anywhere (NFR-006).

**Platforms in v0.8.0.** Windows only. On macOS the button is shown disabled with a hint, until the transport has a thread of its own.

## Consequences

- PRD FR-062 and FR-063 describe this flow. `src-tauri/src/camera.rs` holds the session, `ui/src/features/capture/` the screen, and the capture screen shares its frame with the preview, so the two cannot drift apart.
- A session folder outlives the app until the user saves or deletes it. If the app is reopened, the tab comes back like any other folder, through the autosaved session.
- Moving files is a destructive operation: SECURITY.md covers it (no overwrite, size check before delete), and tests cover it (`fr_063_*`).
- Later: macOS support, other Sony bodies ([cameras/sony.md](../cameras/sony.md), read-only probe), webcam (UVC) live view, and watching a folder for shots from a maker's own app (FR-064).
