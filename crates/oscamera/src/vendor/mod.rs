// SPDX-License-Identifier: GPL-3.0-only
//! One module per camera maker: the maker's own PTP operations, written from our protocol
//! notes in `docs/cameras/<maker>.md` (ADR-0026, clean-room rule). Pure Rust, no I/O; the
//! platform transport sends what these modules build.

pub mod sony;
