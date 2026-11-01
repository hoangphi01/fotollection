// SPDX-License-Identifier: GPL-3.0-only
import { ListChecks } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import type { SelectModeApi } from './use-select-mode'

type SelectModeButtonProps = {
  select: SelectModeApi
}

/** Select / Done (FR-050), the same button in the gallery and in Collage mode. */
export function SelectModeButton({ select }: SelectModeButtonProps) {
  const { t } = useTranslation()
  return (
    <Button
      variant={select.active ? 'default' : 'outline'}
      size="sm"
      aria-pressed={select.active}
      data-hint={t('select.hint')}
      onClick={() => (select.active ? select.leave() : select.start())}
    >
      <ListChecks data-icon="inline-start" />
      <span className="hidden @3xl:inline">{t(select.active ? 'select.done' : 'select.start')}</span>
    </Button>
  )
}
