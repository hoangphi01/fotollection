// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'

/**
 * The bar under the gallery's toolbar (STYLE_RULE §4): select mode's actions (FR-050) and
 * RollSense's (FR-141, owner 2026-11-19: "a second toolbar under the main one"). One line of a
 * fixed 48 px, never wrapping, so nothing under it moves when what it holds changes; its words
 * on the left, its actions at the right, as toolbar buttons that keep only their icons in a
 * narrow window.
 */
export function SubToolbar({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <div className="@container flex h-12 shrink-0 items-center gap-2 overflow-x-auto overflow-y-hidden border-b bg-primary/5 px-4 text-sm [scrollbar-width:thin] [&>*]:shrink-0">
      {label}
      <div className="ml-auto flex items-center gap-2">{children}</div>
    </div>
  )
}
