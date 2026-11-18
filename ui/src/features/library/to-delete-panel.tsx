// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: the right column while the sidebar's *To delete* is chosen (FR-141).
import { HardDrive, Images } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { FactLines, Hint, Stack, ValueRow } from '@/components/forms/field'
import { CommitInput } from '@/components/forms/commit-input'
import { InspectorSection } from '@/components/layout/inspector-section'
import { PanelNote } from '@/components/layout/panel-note'
import { SidePanel } from '@/components/layout/side-panel'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { commands, type PhotoId, type TrashCheck } from '@/ipc/bindings'
import { PanelFooter } from '@/features/about/panel-footer'
import { errorText } from '@/lib/errors'
import { formatBytes, formatNumber } from '@/lib/format'

/**
 * *To delete*'s column (FR-141, owner 2026-11-18, two UX reviews): the photos marked with
 * Delete or from RollSense wait here; the grid ticks the ones to keep and sends them back, and
 * one press moves the rest to the Trash. Built as the settings column is.
 */
export function ToDeletePanel({ shown }: { shown: PhotoId[] }) {
  const { t } = useTranslation()
  const count = shown.length
  const [check, setCheck] = useState<TrashCheck | null>(null)
  const [asking, setAsking] = useState(false)
  const [forGood, setForGood] = useState<PhotoId[] | null>(null)
  const [typed, setTyped] = useState('')
  const [done, setDone] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  // The ids change as a whole when the marked photos do: their count and two ends say so.
  const key = `${count}:${shown[0] ?? ''}:${shown[count - 1] ?? ''}`

  useEffect(() => {
    const ids = shown
    if (ids.length === 0) return setCheck(null)
    void commands.trashCheck(ids).then((result) => {
      if (result.status === 'ok') setCheck(result.data)
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps -- `key` stands for `shown`.
  }, [key])

  const moveAll = async () => {
    setAsking(false)
    setError(null)
    const result = await commands.trashPhotos(shown)
    if (result.status === 'error') return setError(errorText(result.error))
    setDone(result.data.removed)
    if (result.data.notRemoved.length > 0) setForGood(result.data.notRemoved)
  }
  const deleteForGood = async (ids: PhotoId[]) => {
    setForGood(null)
    setTyped('')
    const result = await commands.deletePhotos(ids)
    if (result.status === 'error') setError(errorText(result.error))
    else setDone((d) => (d ?? 0) + result.data.removed)
  }
  const without = check?.withoutTrash ?? 0

  return (
    <SidePanel
      title={t('library.toDelete')}
      footer={
        <>
          {count > 0 && <RemoveButton kind="delete" wide label={t('toDelete.move', { count })} onClick={() => setAsking(true)} />}
          <PanelFooter />
        </>
      }
    >
      {error && <PanelNote danger>{error}</PanelNote>}
      {done !== null && <PanelNote>{t('toDelete.done', { count: done })}</PanelNote>}
      {count === 0 ? (
        <PanelNote>{t('toDelete.empty')}</PanelNote>
      ) : (
        <>
          <Hint>{t('toDelete.hint')}</Hint>
          <InspectorSection icon={<Images />} title={t('toDelete.summary')}>
            <FactLines>
              <ValueRow label={t('toDelete.photos')}>{formatNumber(count)}</ValueRow>
              {check && <ValueRow label={t('toDelete.frees')}>{formatBytes(check.bytes ?? 0)}</ValueRow>}
            </FactLines>
          </InspectorSection>
          {without > 0 && (
            <PanelNote danger icon={<HardDrive />}>
              {t('toDelete.noTrash', { count: without })}
            </PanelNote>
          )}
        </>
      )}
      <AlertDialog open={asking} onOpenChange={setAsking}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('toDelete.deleteTitle', { count })}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('toDelete.deleteBody')}
              {without > 0 && ` ${t('toDelete.noTrash', { count: without })}`}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <RemoveButton kind="delete" label={t('toDelete.deleteConfirm')} onClick={() => void moveAll()} />
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <AlertDialog open={forGood !== null} onOpenChange={(open) => !open && setForGood(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('toDelete.forGoodTitle', { count: forGood?.length ?? 0 })}</AlertDialogTitle>
            <AlertDialogDescription>{t('toDelete.forGoodBody')}</AlertDialogDescription>
          </AlertDialogHeader>
          <Stack>
            <Hint>{t('toDelete.typeCount', { count: forGood?.length ?? 0 })}</Hint>
            <CommitInput value={typed} placeholder={String(forGood?.length ?? "")} numeric liveMs={150} onCommit={setTyped} ariaLabel={t('toDelete.typeCount', { count: forGood?.length ?? 0 })} />
          </Stack>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('toDelete.keep')}</AlertDialogCancel>
            <RemoveButton
              kind="delete"
              label={t('toDelete.forGoodConfirm')}
              disabled={typed.trim() !== String(forGood?.length ?? '')}
              onClick={() => forGood && void deleteForGood(forGood)}
            />
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </SidePanel>
  )
}
