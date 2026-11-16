// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'

/**
 * A preview's row of tools above the stage (FR-095), the photo's and the sheet's alike: one
 * header tall, 8 px between tools; too narrow for every button, the row scrolls sideways
 * rather than cut its last ones (owner, 2026-11-07), and the tools' labels follow its width.
 */
export function ToolbarRow({ children }: { children: ReactNode }) {
  return (
    <div className="@container flex h-header items-center gap-2 overflow-x-auto overflow-y-hidden px-3 [scrollbar-width:thin] [&>*]:shrink-0">
      {children}
    </div>
  )
}
