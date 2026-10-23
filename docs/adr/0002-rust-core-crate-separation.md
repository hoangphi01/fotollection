# ADR-0002: Tauri-independent Rust core crate

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** ADR-0001, [ARCHITECTURE.md §2](../ARCHITECTURE.md#2-repository-layout)

## Context

Image logic must be benchmarked, fuzzed and unit-tested heavily. Coupling it to Tauri types would slow builds and complicate testing. It would also lock the logic to one shell, even though a CLI or another front end might be useful later.

## Options considered

1. **Single crate inside `src-tauri`:** simplest to start. But tests and benchmarks pull in the whole app.
2. **Workspace with `crates/core` (pure) and `src-tauri` (thin shell):** clean boundaries, and the core can be tested and benchmarked on its own.

## Decision

Use a Cargo workspace. `fotollection-core` contains all domain logic and **must not depend on `tauri`**. `src-tauri` only adapts IPC commands, events and protocols to the core API.

## Consequences

- Benchmarks and fuzz targets live in the core crate and build fast.
- A headless CLI (`fotollection-cli`) is cheap to add later.
- CI enforces the boundary: `cargo tree -p fotollection-core` must not contain `tauri`.
