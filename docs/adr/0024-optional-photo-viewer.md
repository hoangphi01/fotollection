# ADR-0024: Fotollection as an optional photo viewer ("Open with")

- **Status:** Accepted
- **Date:** 2026-10-25
- **Related:** FR-054, NFR-016, ADR-0006, ADR-0017, ADR-0023

## Context

People open photos from Finder and Explorer. [FR-054](../PRD.md#fr-054--open-photos-with-fotollection) lets them choose Fotollection there, so it can replace a classic viewer (IrfanView, Photos). Three questions had to be settled:

- **Default or not.** Tauri's file associations are built for documents an app owns. On Windows its NSIS template writes each extension's default value (`Software\Classes\.jpg` → the app's ProgID), so installing would make Fotollection the default viewer for every JPEG; the MSI does the same through WiX.
- **Where the photo goes.** Fotollection works on a library of folders, not on one file.
- **Second launches.** On Windows and Linux, opening a photo while the app runs starts a second process, which would write the same session file as the first.

## Decision

- **Never the default viewer.** On macOS the photo types are declared with the rank `Alternate` and the role `Viewer` (`tauri.macos.conf.json`), so Fotollection appears in "Open With" only. On Windows the photo types are not declared through Tauri: an NSIS hook (`windows/open-with.nsh`) and a WiX fragment (`windows/open-with.wxs`) add a `Fotollection.Photo` ProgID and list it under each type's `OpenWithProgids`, without touching the default; the uninstaller removes them. The project file keeps its own association as its owner.
- **The photo's folder becomes a tab.** The opened photo's folder is imported **without its subfolders** (a photo in Downloads or on the Desktop must not walk everything below it), unless the photo is already in the library, and the preview opens on it; ← → browse its folder. Its edits, basket and export work as for any folder, and × closes the tab.
- **One instance.** `tauri-plugin-single-instance` (MIT/Apache-2.0) hands a second launch's paths to the running app, which comes to the front.
- **One entry point.** macOS `Opened` events, command-line paths at start-up and second launches all go through `open_with::route`: a project to the document flow, anything else to `open_photo`, which checks the file like an import (magic bytes, size, polyglot) before touching the library and waits for the last session to be restored first.
- **No lost request.** The request is kept in Rust until the UI takes it (`take_photo_request`), on the `OpenPhoto` event and after `StartupFinished`; the UI opens the preview once that folder's photos are listed.

## Consequences

- Choosing Fotollection as the default stays the user's own decision, in the system's settings.
- Opening a photo adds a tab to the session and marks it edited, like opening a folder.
- The Windows registry entries are written by our hook and fragment, so a change of Tauri's installer templates can affect them; they are checked by hand on Windows before each release that touches them.
