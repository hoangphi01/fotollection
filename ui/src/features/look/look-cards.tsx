// SPDX-License-Identifier: GPL-3.0-only
import type { RenderSettings } from '@/ipc/bindings'
import { FavesCard } from './faves-card'
import { ToolsCard } from './tools-card'
import { fullLook, type LookChange } from './look-fields'

type LookCardsProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * The look of every photo without one of its own (FR-079): Faves and Tools, between the
 * print size and the date stamp in every panel (STYLE_RULE §4, same tools, same order).
 * The shared setting has no undo history; a drag's values replace each other.
 */
export function LookCards({ settings, onChange }: LookCardsProps) {
  const change: LookChange = (edit) => onChange((s) => ({ ...s, look: edit(fullLook(s.look)) }))
  return (
    <>
      <FavesCard look={settings?.look} onChange={change} />
      <ToolsCard look={settings?.look} onChange={change} />
    </>
  )
}
