// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { RectangleHorizontal, RectangleVertical } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { commands, type PhotoId } from '@/ipc/bindings'
import { loadPrintGeometry } from './turn-print-size'
import { useLiveQuery } from './use-ipc-query'

type Orientation = 'upright' | 'flat'


/**
 * The print size *Upright* or *Flat* for these photos (FR-120): a portrait photo laid across
 * a 15 × 5 strip, a landscape one stood up. Rust decides (`plan.rs`); this only shows what the
 * first photo prints as and asks for the other way. With several photos ticked nothing is
 * shown chosen, and a choice lays each one that way. Square sizes, Original and the frame's
 * Fit have nothing to turn.
 */
export function PrintOrientationField({ photoIds, revision }: { photoIds: readonly PhotoId[]; revision: number }) {
  const { t } = useTranslation()
  const first = photoIds[0]
  const { data: geometry } = useLiveQuery(first === undefined ? null : String(first), revision, loadPrintGeometry)
  if (first === undefined) return null
  const single = photoIds.length === 1
  const fixed = geometry !== null && !geometry.canTurn
  const value: Orientation | undefined =
    single && geometry ? (geometry.printLandscape ? 'flat' : 'upright') : undefined
  const hint = fixed
    ? t('printTurn.fixed')
    : single
      ? t('printTurn.hint')
      : t('printTurn.hintMany', { count: photoIds.length })
  return (
    <Field label={t('printTurn.label')} hint={hint}>
      <SegmentedToggle<Orientation>
        value={value}
        options={[
          { value: 'upright', label: <RectangleVertical aria-hidden />, ariaLabel: t('printTurn.upright'), hint: t('printTurn.upright') },
          { value: 'flat', label: <RectangleHorizontal aria-hidden />, ariaLabel: t('printTurn.flat'), hint: t('printTurn.flat') },
        ]}
        onChange={(orientation) => void commands.setPrintOrientation([...photoIds], orientation === 'flat')}
        ariaLabel={t('printTurn.label')}
        disabled={fixed}
      />
    </Field>
  )
}
