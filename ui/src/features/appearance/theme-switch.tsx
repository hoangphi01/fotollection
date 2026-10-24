// SPDX-License-Identifier: GPL-3.0-only
import { Monitor, Moon, Sun } from 'lucide-react'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
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

  return (
    <ToggleGroup
      type="single"
      size="sm"
      variant="outline"
      value={ui.theme}
      onValueChange={(v) => v && void update({ theme: v as ThemePreference })}
      aria-label={t('theme.label')}
    >
      {OPTIONS.map(({ value, Icon }) => (
        <Tooltip key={value}>
          <TooltipTrigger asChild>
            <ToggleGroupItem
              value={value}
              aria-label={t(`theme.${value}`)}
              className="px-2"
            >
              <Icon className="size-3.5" />
            </ToggleGroupItem>
          </TooltipTrigger>
          <TooltipContent>{t(`theme.${value}`)}</TooltipContent>
        </Tooltip>
      ))}
    </ToggleGroup>
  )
}
