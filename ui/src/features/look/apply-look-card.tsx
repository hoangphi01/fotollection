// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: the gallery's Colour tab only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { CircleCheck, Layers, OctagonX, Palette, Wand } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton, ResetButton } from '@/components/forms/danger-button'
import { SliderField } from '@/components/forms/slider-field'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { commands, type CullFilter, type LookEntry, type PhotoSummary } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { formatNumber } from '@/lib/format'
import { lookTileUrl } from '@/lib/photo-url'
import { useLookLibrary } from './look-library'
import { lookName, useLooks } from './use-looks'

type ApplyLookCardProps = {
  /** The gallery's filter: the photos a batch covers when none is ticked. */
  filter: CullFilter
  /** The ticked photos in select mode (FR-050), which a batch covers instead. */
  ticked?: number[]
  /** How many photos the gallery shows. */
  photoCount: number
  /** The photo picked in the grid: the look is shown on it, and its look can be saved. */
  photo?: PhotoSummary
}

/** The look chosen, kept while the app runs (a tab switched, the panel shown again). */
let lastChosen: string | undefined
let lastAmount = 100

/** What the last batch did, until the next: a look given, or the photos back to their original colour. */
type Done = { name: string; applied: number; kept: number; reset: boolean } | null

/** The question asked before a batch: a look over photos with their own colour, or a reset. */
type Asking = { mode: 'apply' | 'reset'; edited: number; total: number } | null

/** The library's look that is the photo's own colour, untouched (FR-079). */
const ORIGINAL = 'builtin:original'

const percent = (v: number) => `${v} %`

/**
 * The gallery's Colour tab (FR-114, owner, 2026-11-08: "Lightroom applies a preset only at
 * import"): a look, LUT or preset of the library, seen on the picked photo, given to the ticked
 * photos or else to every photo shown, at an amount; photos with a colour of their own are
 * asked about once. Rust picks the photos and builds the look; one ⌘Z undoes the batch.
 */
export function ApplyLookCard({ filter, ticked, photoCount, photo }: ApplyLookCardProps) {
  const { t } = useTranslation()
  const { looks, setLooks, reload } = useLooks()
  const [chosenId, setChosenId] = useState(lastChosen)
  const [amount, setAmount] = useState(lastAmount)
  const [asking, setAsking] = useState<Asking>(null)
  const [done, setDone] = useState<Done>(null)
  const [error, setError] = useState<string | null>(null)
  const chosen = looks.find((e) => e.id === chosenId)
  const pick = (entry: LookEntry) => {
    lastChosen = entry.id
    setChosenId(entry.id)
    setDone(null)
  }
  const library = useLookLibrary({
    looks,
    reload,
    remove: (id) => void commands.deleteLook(id).then((r) => r.status === 'ok' && setLooks(r.data)),
    chosenId,
    onPick: pick,
    photoId: photo?.id,
    builtIns: true,
    scroll: true,
  })
  const tickedIds = ticked ?? []
  const count = tickedIds.length > 0 ? tickedIds.length : photoCount

  const original = looks.find((e) => e.id === ORIGINAL)
  // One way for both: a look given, or *Original* given back at full amount (the reset, owner
  // 2026-11-08); Rust picks the photos and makes it one ⌘Z step.
  const run = (mode: 'apply' | 'reset', keepEdited: boolean) => {
    const entry = mode === 'reset' ? original : chosen
    if (!entry) return
    setAsking(null)
    const name = lookName(entry)
    void commands
      .applyLookToPhotos({ filter, ticked: tickedIds, look: entry.id, amount: mode === 'reset' ? 100 : amount, name, keepEdited })
      .then((r) => {
        if (r.status !== 'ok') return setError(errorText(r.error))
        setError(null)
        setDone({ name, applied: r.data.applied, kept: r.data.kept, reset: mode === 'reset' })
      })
  }
  const start = () => {
    if (!chosen) return
    void commands.lookBatchPlan(filter, tickedIds, chosen.id).then((r) => {
      if (r.status !== 'ok') return setError(errorText(r.error))
      if (r.data.edited > 0) setAsking({ mode: 'apply', edited: r.data.edited, total: r.data.total })
      else run('apply', false)
    })
  }
  // A reset always asks: it takes every photo's colour away at once.
  const askReset = () =>
    void commands.lookBatchPlan(filter, tickedIds, ORIGINAL).then((r) => {
      if (r.status !== 'ok') return setError(errorText(r.error))
      setAsking({ mode: 'reset', edited: r.data.edited, total: r.data.total })
    })

  return (
    <>
    {/* What is given, to how many, first, so the button is in sight; the library under it. */}
    <SectionCard
      icon={<Wand />}
      title={t('look.applyTitle')}
      action={
        <ResetButton
          iconOnly
          label={t(tickedIds.length > 0 ? 'look.resetTicked' : 'look.resetShown', { count, n: formatNumber(count) })}
          disabled={count === 0 || !original}
          onClick={askReset}
        />
      }
    >
      {chosen && (
        <>
          {/* The chosen look on the picked photo, Rust's small tile (as the strips under the photo). */}
          {photo && (
            <div className="flex items-center gap-2">
              <img
                src={lookTileUrl(photo.id, chosen.id)}
                alt=""
                draggable={false}
                className="size-24 shrink-0 rounded-md bg-muted object-cover raised"
              />
              <p className="min-w-0 text-sm">
                <span className="block truncate font-medium" data-hint={lookName(chosen)}>
                  {lookName(chosen)}
                </span>
                <span className="block text-xs text-muted-foreground">{t('look.applyPreviewOn')}</span>
              </p>
            </div>
          )}
          <SliderField
            label={t('look.amount')}
            hint={t('look.hints.amount')}
            value={amount}
            min={0}
            max={100}
            reset={100}
            format={percent}
            gpu={false}
            onChange={(v) => {
              lastAmount = v
              setAmount(v)
            }}
          />
          <Button className="w-full" disabled={count === 0} onClick={start}>
            <Wand data-icon="inline-start" />
            {t(tickedIds.length > 0 ? 'look.applyTicked' : 'look.applyShown', { count, n: formatNumber(count) })}
          </Button>
        </>
      )}
      {!chosen && <PanelNote icon={<Palette />}>{t('look.applyPick')}</PanelNote>}
      {done && (
        <PanelNote icon={<CircleCheck />}>
          {done.reset
            ? t('look.resetDone', { count: done.applied, n: formatNumber(done.applied) })
            : t('look.applyDone', { name: done.name, count: done.applied, n: formatNumber(done.applied) })}
          {done.kept > 0 && ` ${t('look.applyKept', { count: done.kept, n: formatNumber(done.kept) })}`}
        </PanelNote>
      )}
      {error && <PanelNote danger icon={<OctagonX />}>{error}</PanelNote>}
      <AlertDialog open={asking !== null} onOpenChange={(open) => !open && setAsking(null)}>
        <AlertDialogContent>
          {/* One dialog for both questions (owner, 2026-11-08: "reuse the apply pop-up"). */}
          <AlertDialogHeader>
            <AlertDialogTitle>
              {asking?.mode === 'reset'
                ? t('look.resetAskTitle', { count: asking.total, n: formatNumber(asking.total) })
                : t('look.applyAskTitle', { count: asking?.edited ?? 0, n: formatNumber(asking?.edited ?? 0) })}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {asking?.mode === 'reset' ? t('look.resetAskBody') : t('look.applyAskBody', { name: chosen ? lookName(chosen) : '' })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            {asking?.mode === 'reset' ? (
              // Its action is the red one, so Cancel stays neutral: one red, the danger.
              <>
                <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
                <ResetButton label={t('look.resetOk')} onClick={() => run('reset', false)} />
              </>
            ) : (
              <>
                <RemoveButton kind="stop" label={t('common.cancel')} onClick={() => setAsking(null)} />
                <Button variant="outline" onClick={() => run('apply', true)}>
                  {t('look.applyKeepEdited')}
                </Button>
                <AlertDialogAction onClick={() => run('apply', false)}>
                  {t('look.applyOverwrite', { count: asking?.total ?? 0, n: formatNumber(asking?.total ?? 0) })}
                </AlertDialogAction>
              </>
            )}
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </SectionCard>
    <SectionCard icon={<Layers />} title={t('look.presets')} action={library.action}>
      {library.content}
    </SectionCard>
    </>
  )
}
