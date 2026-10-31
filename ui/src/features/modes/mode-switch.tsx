// SPDX-License-Identifier: GPL-3.0-only
import { Columns2, SlidersHorizontal, Zap } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { AppMode } from '@/ipc/bindings'
import { useAppMode } from './use-app-mode'

const MODES: { mode: AppMode; icon: typeof Zap }[] = [
  { mode: 'simple', icon: Zap },
  { mode: 'collage', icon: Columns2 },
  { mode: 'professional', icon: SlidersHorizontal },
]

/** The three modes, side by side at the top of the sidebar (FR-048). */
export function ModeSwitch() {
  const { t } = useTranslation()
  const { mode, setMode } = useAppMode()
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      size="sm"
      value={mode}
      onValueChange={(v) => v && setMode(v as AppMode)}
      aria-label={t('mode.label')}
      className="w-full"
    >
      {MODES.map(({ mode: m, icon: Icon }) => (
        <ToggleGroupItem key={m} value={m} className="flex-1 gap-1.5 px-2 text-xs" data-hint={t(`mode.${m}Hint`)}>
          <Icon />
          {t(`mode.${m}`)}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}
