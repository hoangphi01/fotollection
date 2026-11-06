// SPDX-License-Identifier: GPL-3.0-only
import { Fragment, type ReactNode } from 'react'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { Separator } from '@/components/ui/separator'

/** One tab: its icon, and its name and key for the hint and screen readers. */
export type InspectorTab<T extends string> = { value: T; icon: ReactNode; label: string }

type InspectorTabsProps<T extends string> = {
  /** Groups of tabs, a separator between them: the tools that change the stage (Crop, later
   * Heal and Mask, FR-101/102), then the panels (Print, Info), as Lightroom's tool strip sits
   * apart from its panels. A tab is added to a list, never to the layout. */
  groups: readonly (readonly InspectorTab<T>[])[]
  value: T
  onChange: (value: T) => void
  ariaLabel: string
}

/**
 * The row of icon tabs at the head of a preview's inspector (FR-095): icons with their words
 * as the hint (STYLE_RULE §4, icons first), the open one pressed in (FR-099), the inspector
 * staying 18 rem.
 */
export function InspectorTabs<T extends string>({ groups, value, onChange, ariaLabel }: InspectorTabsProps<T>) {
  return (
    <div role="tablist" aria-label={ariaLabel} className="flex items-center gap-2 p-3">
      {groups.map((tabs, i) => (
        <Fragment key={tabs[0]?.value ?? i}>
          {i > 0 && <Separator orientation="vertical" className="h-5!" />}
          <SegmentedToggle<T>
            small
            value={tabs.some((t) => t.value === value) ? value : undefined}
            ariaLabel={ariaLabel}
            options={tabs.map((t) => ({ value: t.value, label: t.icon, ariaLabel: t.label, hint: t.label }))}
            onChange={onChange}
          />
        </Fragment>
      ))}
    </div>
  )
}
