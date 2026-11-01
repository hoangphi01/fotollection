// SPDX-License-Identifier: GPL-3.0-only
import { useState, type SyntheticEvent } from 'react'

/**
 * How a sheet's halves sit, read from its image once loaded: side by side on a landscape
 * sheet, one above the other on a standing one (FR-035). `loaded` says whether any render of
 * this sheet is in: a new setting keeps the last one on screen, without a spinner.
 */
export function useSheetShape(src: string) {
  const [shape, setShape] = useState<{ src: string; sideBySide: boolean } | null>(null)
  return {
    sideBySide: shape?.sideBySide ?? true,
    loaded: shape !== null,
    onLoad: (e: SyntheticEvent<HTMLImageElement>) =>
      setShape({ src, sideBySide: e.currentTarget.naturalWidth >= e.currentTarget.naturalHeight }),
  }
}
