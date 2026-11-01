// SPDX-License-Identifier: GPL-3.0-only
import { RotateCcw, WandSparkles } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { commands, type CullFilter } from '@/ipc/bindings'

/**
 * Collage mode's Auto turn and Revert (FR-035): turn every photo shown so it lies the way its
 * cell does, for prints cut in two, or bring every photo back to its own orientation. Each is
 * one undoable step; the sheets refresh from the change events.
 */
export function AutoTurnButtons({ filter }: { filter: CullFilter }) {
  const { t } = useTranslation()
  return (
    <>
      <Button
        variant="outline"
        size="sm"
        data-hint={t('collage.autoTurnHint')}
        onClick={() => void commands.autoTurnCollage(filter, false)}
      >
        <WandSparkles data-icon="inline-start" />
        <span className="hidden @3xl:inline">{t('collage.autoTurn')}</span>
      </Button>
      <Button
        variant="outline"
        size="sm"
        data-hint={t('collage.revertTurnsHint')}
        onClick={() => void commands.autoTurnCollage(filter, true)}
      >
        <RotateCcw data-icon="inline-start" />
        <span className="hidden @3xl:inline">{t('collage.revertTurns')}</span>
      </Button>
    </>
  )
}
