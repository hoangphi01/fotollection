// SPDX-License-Identifier: GPL-3.0-only
import type { PostcardSettings } from '@/ipc/bindings'

/** As `PostcardSettings::default()` in Rust, for a settings file from before FR-073. */
export const POSTCARD_DEFAULTS: PostcardSettings = {
  enabled: false,
  orientation: 'landscape',
  lines: 4,
  font: null,
  credit: '',
  ink: 'auto',
}
