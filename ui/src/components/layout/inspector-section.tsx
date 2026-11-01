// SPDX-License-Identifier: GPL-3.0-only
import type { ReactNode } from 'react'
import { SectionCard } from './section-card'

type InspectorSectionProps = {
  icon: ReactNode
  title: string
  children: ReactNode
}

/** A section of a preview's inspector: a card with tighter rows (the photo and the sheet). */
export function InspectorSection({ icon, title, children }: InspectorSectionProps) {
  return (
    <SectionCard icon={icon} title={title}>
      <div className="space-y-1">{children}</div>
    </SectionCard>
  )
}
