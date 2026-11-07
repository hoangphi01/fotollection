// SPDX-License-Identifier: GPL-3.0-only
import { Monitor, Moon, Sun } from 'lucide-react'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import type { ThemePreference } from '@/ipc/bindings'
import { setThemePreference } from '@/lib/theme'

const OPTIONS: { value: ThemePreference; Icon: typeof Sun }[] = [
  { value: 'system', Icon: Monitor },
  { value: 'light', Icon: Sun },
  { value: 'dark', Icon: Moon },
]

/** System / light / dark, stored per machine (FR-052). */
export function ThemeSwitch() {
  const { t } = useTranslation()
  const { ui, update } = useUiSettings()

  useEffect(() => setThemePreference(ui.theme), [ui.theme])

  // The same block as Settings → General's (STYLE_RULE §4: one control, one look), icons
  // with their names as hints, as wide as its items at the foot of the folder list.
  return (
    <SegmentedToggle<ThemePreference>
      fit
      value={ui.theme}
      ariaLabel={t('theme.label')}
      onChange={(theme) => void update({ theme })}
      options={OPTIONS.map(({ value, Icon }) => ({
        value,
        label: <Icon aria-hidden />,
        ariaLabel: t(`theme.${value}`),
        hint: t(`theme.${value}`),
      }))}
    />
  )
}
