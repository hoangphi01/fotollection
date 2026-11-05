// SPDX-License-Identifier: GPL-3.0-only
import { RotateCcw, WandSparkles } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { commands, type CullFilter } from '@/ipc/bindings'

/**
 * Collage mode's Auto turn and Revert (FR-035): turn every photo shown so it lies the way its
 * cell does, for prints cut in two, or bring every photo back to its own orientation. Each is
 * one undoable step; the sheets refresh from the change events. While one runs, its button
 * spins and neither can be pressed again.
 */
export function AutoTurnButtons({ filter }: { filter: CullFilter }) {
  const { t } = useTranslation()
  const [running, setRunning] = useState<'turn' | 'revert' | null>(null)
  const run = (which: 'turn' | 'revert') => {
    setRunning(which)
    void commands.autoTurnCollage(filter, which === 'revert').finally(() => setRunning(null))
  }
  return (
    <>
      <ToolbarButton
        icon={<WandSparkles />}
        label={t('collage.autoTurn')}
        data-hint={t('collage.autoTurnHint')}
        busy={running === 'turn'}
        disabled={running !== null}
        onClick={() => run('turn')}
      />
      <ToolbarButton
        icon={<RotateCcw />}
        label={t('collage.revertTurns')}
        data-hint={t('collage.revertTurnsHint')}
        busy={running === 'revert'}
        disabled={running !== null}
        onClick={() => run('revert')}
      />
    </>
  )
}
