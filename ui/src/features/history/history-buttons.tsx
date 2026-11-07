// SPDX-License-Identifier: GPL-3.0-only
import { Redo2, Undo2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { commands } from '@/ipc/bindings'
import { MOD } from '@/lib/platform'

/**
 * Undo and redo, first in a viewer's toolbar (owner, 2026-11-07: they were taken for the turn
 * buttons): one block for the photo preview and the sheet preview, so both read the same.
 */
export function HistoryButtons({ canUndo, canRedo }: { canUndo: boolean; canRedo: boolean }) {
  const { t } = useTranslation()
  return (
    <>
      <ToolbarButton
        icon={<Undo2 />}
        label={t('viewer.undo')}
        data-hint={t('viewer.undoHint', { mod: MOD })}
        iconOnly
        disabled={!canUndo}
        onClick={() => void commands.undo()}
      />
      <ToolbarButton
        icon={<Redo2 />}
        label={t('viewer.redo')}
        data-hint={t('viewer.redoHint', { mod: MOD })}
        iconOnly
        disabled={!canRedo}
        onClick={() => void commands.redo()}
      />
    </>
  )
}
