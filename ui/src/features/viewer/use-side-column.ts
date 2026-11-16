// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'

/** Under this window width, three columns leave the photo too small (FR-095). */
const WIDE_WINDOW = 1400

/**
 * A preview's left column (FR-095), the photo's and the sheet's alike (owner, 2026-11-16:
 * "learn from Pro, reuse"): beside the stage in a wide window; a narrow one folds it, and the
 * toolbar's button or Tab brings it back. `enabled` is the layout that has the column, `panels`
 * whether Tab has hidden the side columns.
 */
export function useSideColumn(enabled: boolean, panels = true) {
  const [wide, setWide] = useState(() => window.innerWidth >= WIDE_WINDOW)
  const [open, setOpen] = useState(false)
  useEffect(() => {
    const onResize = () => setWide(window.innerWidth >= WIDE_WINDOW)
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])
  const narrow = enabled && !wide
  return {
    /** The column is shown. */
    show: enabled && panels && (wide || open),
    /** A narrow window folds it: the toolbar's button and the column's own ✕ show. */
    narrow,
    setOpen,
    /** The toolbar's button, while a narrow window keeps the column folded. */
    toggle: narrow && !open ? { open: false, onToggle: () => setOpen(true) } : undefined,
  }
}
