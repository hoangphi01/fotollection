// SPDX-License-Identifier: GPL-3.0-only
import type { DpiBand, PrintVerdict } from '@/ipc/bindings'

/** The inspector's DPI band in the three steps of the badges (mirrors `PrintVerdict::from`). */
export function verdictOf(band: DpiBand): PrintVerdict {
  if (band === 'excellent') return 'sharp'
  return band === 'poor' ? 'pixelated' : 'acceptable'
}
