// SPDX-License-Identifier: GPL-3.0-only
import { ListChecks } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import type { SelectModeApi } from './use-select-mode'

type SelectModeButtonProps = {
  select: SelectModeApi
}

/** Select / Done (FR-050), the same button in the gallery and in Collage mode. */
export function SelectModeButton({ select }: SelectModeButtonProps) {
  const { t } = useTranslation()
  return (
    <ToolbarButton
      icon={<ListChecks />}
      label={t(select.active ? 'select.done' : 'select.start')}
      variant={select.active ? 'default' : 'outline'}
      aria-pressed={select.active}
      data-hint={t('select.hint')}
      onClick={() => (select.active ? select.leave() : select.start())}
    />
  )
}
