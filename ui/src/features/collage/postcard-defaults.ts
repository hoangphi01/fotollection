// SPDX-License-Identifier: GPL-3.0-only
import type { PostcardSettings, RenderSettings } from '@/ipc/bindings'

/** As `PostcardSettings::default()` in Rust, for a settings file from before FR-073. */
export const POSTCARD_DEFAULTS: PostcardSettings = {
  enabled: false,
  orientation: 'landscape',
  lines: 4,
  font: null,
  credit: '',
  ink: 'auto',
  textSize: 'medium',
  messageStyle: { bold: false, italic: false, underline: false },
  addressStyle: { bold: false, italic: false, underline: false },
}

/** Patches a postcard setting, from a file written before FR-073 too. */
export function patchPostcard(
  onChange: (change: (current: RenderSettings) => RenderSettings) => void,
  patch: Partial<PostcardSettings>,
) {
  onChange((s) => ({
    ...s,
    collage: { ...s.collage, postcard: { ...POSTCARD_DEFAULTS, ...s.collage?.postcard, ...patch } },
  }))
}
