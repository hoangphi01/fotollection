// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'

export type ColourPrintTab = 'colour' | 'print'

/** Each panel's open tab, kept while the app runs; Print first, as the panels always opened. */
const kept = new Map<string, ColourPrintTab>()

/** A panel's tab (`ColourPrintTabs`), remembered under the panel's own key. */
export function useColourPrintTab(panel: 'gallery' | 'collage'): [ColourPrintTab, (tab: ColourPrintTab) => void] {
  const [tab, setTab] = useState<ColourPrintTab>(() => kept.get(panel) ?? 'print')
  const open = (next: ColourPrintTab) => {
    kept.set(panel, next)
    setTab(next)
  }
  return [tab, open]
}
