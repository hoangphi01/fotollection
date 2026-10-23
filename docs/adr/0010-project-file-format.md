# ADR-0010: Project file format for culling and edits

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** FR-022, FR-023, FR-024, FR-025, FR-030, FR-032, NFR-013, ADR-0006

## Context

Culling ([FR-022](../PRD.md#fr-022--culling-rate-flag-filter)) and per-photo edits (sizes, crops, clock offsets, manual places) represent hours of user work, so they must survive restarts and crashes. The sources must stay read-only ([SECURITY.md §4](../SECURITY.md#4-file-system-safety)). Users may also move a project between machines (macOS and Windows).

## Options considered

1. **Write ratings to source files (EXIF/XMP in place):** interoperable with Lightroom, but it modifies the originals and risks corrupting them. This breaks a core promise.
2. **XMP sidecar files next to each photo:** interoperable, but scatters hundreds of files across the user's folders, and it is awkward for app-specific data (print sizes, borders, presets).
3. **Embedded database (SQLite):** robust and queryable, but adds a native dependency, a binary format that is hard to diff, and schema migrations of its own.
4. **One versioned JSON project file (`.fotollection`):** simple, human-readable, easy to back up and move, and enough for tens of thousands of photos.

## Decision

Store all working state in a single **versioned JSON project file**:

- The top-level `schema_version` is an integer. Migrations run forward on open, and a newer version than the app understands is refused, never overwritten.
- Photos are keyed by a stable ID derived from the path relative to their source folder. Source folders are stored relative to the project file when possible, so the project still opens after moving to another machine.
- Writes are **atomic**: serialize, write a temp file in the same directory, fsync, then rename. Autosave is debounced to at most every 5 s ([NFR-013](../PRD.md#nfr-013--project-safety)). The previous version is kept as `.fotollection.bak`.
- Undo/redo is an in-memory command stack and is **not** stored in the file.

## Consequences

- Sources are never touched. XMP sidecar **export** stays a backlog idea for interoperability.
- The project file is untrusted input when opened. It is size-capped, parsed with a typed schema, and its paths are validated ([SECURITY.md §3](../SECURITY.md#3-input-and-metadata-defense)).
- For very large libraries, re-serializing the whole file costs some milliseconds. If benchmarks show a problem, revisit (for example with an append-only journal) in a superseding ADR.
