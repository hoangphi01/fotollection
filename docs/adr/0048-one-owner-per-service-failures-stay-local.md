# ADR-0048: One owner per service; a failing part degrades, never takes the app down

- **Status:** Accepted
- **Date:** 2026-11-09
- **Related:** NFR-011, NFR-013, FR-023, FR-082, FR-116, ADR-0029, ADR-0044

## Context

An audit of 2026-11-09 (the owner: "treat the app as a small Amazon ecosystem: the right job to the right owner, nothing overlapping; one block or service that fails must not take the app down") found that most failures were already contained (no `unwrap` outside tests, panics caught around rawler, per-photo results in batches), but four were not:

1. A panic while serving an image aborts the process: every image request runs as a rayon `spawn` on the interactive pool, which has no `panic_handler`, and rayon aborts on an unhandled panic in a spawned job. JPEG, PNG, EXIF and RAW-preview decoding are not inside `catch_unwind`.
2. A panic in a synchronous Tauri command (96 of them run on the main thread) quits the app.
3. A React render error anywhere unmounts the whole tree: there is no error boundary.
4. A font or logo that panics its decoder at start-up stops the app from starting, every time.

It also found failures that leave the app stuck without a word (a dead export or analysis thread keeps the UI "running" for good; a poisoned `library` lock fails every later command and stops autosave; no worker is ever restarted) and data faults (one bad field resets every setting; projects do not carry their LUTs; opening a project overwrites the machine's settings).

## Options considered

- **Fix each site as found:** what was done so far; each new service brings the same gaps back.
- **One rule per kind of failure, owned by one helper, checked by a test:** chosen.
- **Abort and restart the process on any panic:** loses the user's unsaved state and the open dialogs; rejected.

## Decision

The app is a set of services, each with **one owner**: the module that holds its data, the thread or pool it runs on, how it fails, its `ERR-` codes and its budget. No service writes another's data; it asks the owner.

| Service | Owner | Data | Runs on | When it fails |
|---|---|---|---|---|
| Library | `library.rs`, `commands.rs` | photos and edits in memory | commands | that command gets an error; the lock heals |
| Document | `document.rs`, `project.rs`, `bundle.rs` | session, `.fotollection` | commands, autosave | the last good file and its `.bak` kept; the user told |
| Machine settings | `settings_store.rs` | `settings.json` | autosave | a bad part resets alone; `.bak` tried first |
| Assets | `lut.rs`, `fonts.rs`, `logo.rs` | `looks/`, `fonts/`, `logos/` | commands | a missing asset is reported, never dropped silently |
| Image server | `protocol.rs` | none | interactive pool | that image answers 500 with its `ERR-` code |
| Caches | `thumbs.rs`, `preview.rs`, `cache.rs` | `thumbs/`, `analysis/` | background pool | a miss is made again |
| Jobs: export, analysis, camera, updates | `export.rs`, `culling.rs`, `camera.rs`, `updates.rs` | the job | supervised threads | the job ends with an error; the UI is never left running |
| Interface | each panel, card and dialog | none | webview | that part shows "This part failed", the rest goes on |

**One helper per kind of failure:**

- **Pools:** every rayon pool has a `panic_handler` that logs `ERR-RUS-0005`; nothing aborts.
- **Image routes:** each route runs inside `catch_unwind`; a panic answers 500 with an `IpcError`.
- **Commands:** the invoke handler runs every command inside `catch_unwind`; a panic answers the call with `ERR-RUS-0005`.
- **Locks:** state locks are taken through one `Locked` helper that recovers a poisoned lock's data and logs `ERR-RUS-0003` once; a test refuses the old patterns.
- **Threads:** every thread starts through `supervise`: a panic is logged, the job's end event is always sent and its entry removed, and a long-lived worker (autosave, memory watcher, prefetch, camera) restarts with a back-off.
- **Decoders:** every decode of an untrusted file (JPEG, PNG, EXIF, RAW previews, fonts, logos, LUTs, presets) goes through one `guarded` call.
- **Start-up:** settings are read part by part (a bad part resets alone, the `.bak` is tried before a quarantine), fonts and logos are skipped when they fail, and the user is told once.
- **Interface:** an error boundary around the shell, each panel, each card (`SectionCard`), each dialog and the viewer, with one fallback block (`PartFailed`); uncaught errors and rejected promises go to the error log through Rust (`ERR-UI-0001`).

Each path is proven by a fault injected in development builds only (never in a release, SECURITY §9), failing on the old code first.

## Consequences

- A corrupt photo, font or setting costs that photo, font or setting, never the session.
- New services follow the table: a new row here, its helpers, its codes in ERROR_CONTROL, its budget in OPTIMIZATION.
- Opening a project no longer overwrites the machine's settings: a project keeps its own (S4 of this plan), the machine keeps the defaults for new projects.
- A little code at each boundary; the helpers keep it to one line a site.
