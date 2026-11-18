# ADR-0055: Deleting to the Trash on the user's word

- **Status:** Accepted
- **Date:** 2026-11-17
- **Related:** FR-143, FR-141, FR-041, FR-050, ADR-0054

## Context

Fotollection has never deleted a user's photo: culling flags, *Set aside* hides, and ADR-0054 says a sorted roll's files are never changed, renamed or deleted. A phone's roll is different from a shoot. Of the owner's 14,639 files, thousands are soft frames, screenshots of soft photos, memes and images saved from the web. Setting them aside keeps them on the disk and in every later import. The owner asked on 2026-11-17 to delete them from the app, with good suggestions.

## Options considered

1. **Set aside only, with a list the user deletes elsewhere.** Nothing changes in the app's guarantees, but the job is left half done, and a list of names is hard to act on in Finder or on a NAS.
2. **Delete for good.** One step, and nothing to undo if a suggestion was wrong.
3. **Move to the system's Trash, after the user confirms.** It can be undone where users already look. A volume with no Trash, such as some NAS shares, asks again before deleting for good.

## Decision

Option 3. Suggested deletions (FR-143) carry their reasons and certainty. The user confirms a group or a photo, never a batch they did not see. The files go to the system's Trash through the `trash` crate (MIT: macOS, Windows, FreeDesktop Linux). Paths come from the open library and are checked in Rust, never taken from the webview (SECURITY.md). Nothing is deleted on its own. Culling (FR-022, FR-046) and *Set aside* (FR-050) still never touch a file.

## Consequences

- ADR-0054's "never deleted" now reads "never deleted on its own"; FR-141 says the same.
- A new dependency (`trash`, MIT) and a new error code for a file that cannot be moved to the Trash, both added with the code.
- A wrong suggestion now costs more, so FR-143's suggestions must be right at least 95 % of the time on the owner's labels before they ship.

## Amendment, 2026-11-18: one *To delete* area, emptied at once

Two UX reviews found that RollSense's *Delete n photos* acted on every photo the grid showed, with no way to keep a few and no undo, against this record's "never a batch they did not see". Owner's choice: every delete now only **marks** photos (`queue: delete`, undoable, FR-050), and the sidebar's *To delete* is the one place files leave: its column shows the count, the space freed and the photos on a drive that may have no Trash (checked before the move, `trash_check`); one confirmation moves them all, with a progress bar and a stop (`cancel_trash`); a drive without a Trash then asks for the count to be typed before deleting for good, and `delete_photos` takes only files the Trash refused just before.
