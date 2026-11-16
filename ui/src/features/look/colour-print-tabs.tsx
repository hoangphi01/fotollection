// SPDX-License-Identifier: GPL-3.0-only
import { Palette, Printer } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { InspectorTabs } from '@/components/layout/inspector-tabs'
import type { ColourPrintTab } from './use-colour-print-tab'

/**
 * A gallery panel's two tabs, one joined group (owner, 2026-11-08 and 2026-11-15): **Colour**,
 * a look, LUT or preset given to many photos (FR-114, FR-137), and **Print**. The same tabs in
 * Pro mode's panel and Collage mode's.
 */
export function ColourPrintTabs({ value, onChange }: { value: ColourPrintTab; onChange: (tab: ColourPrintTab) => void }) {
  const { t } = useTranslation()
  return (
    <InspectorTabs<ColourPrintTab>
      groups={[
        [
          { value: 'colour', icon: <Palette />, label: t('inspector.colour') },
          { value: 'print', icon: <Printer />, label: t('inspector.print') },
        ],
      ]}
      value={value}
      onChange={onChange}
      ariaLabel={t('inspector.tabs')}
    />
  )
}
