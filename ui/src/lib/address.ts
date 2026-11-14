// SPDX-License-Identifier: GPL-3.0-only
import type { Address, Country } from '@/ipc/bindings'

/** An empty address of a country, as `Address::new` in Rust (FR-133). */
export function emptyAddress(country: Country): Address {
  return { country, filledBy: '', code: '', region: '', city: '', town: '', road: '', line: '', name: '', honorific: true, latin: false }
}
