// SPDX-License-Identifier: GPL-3.0-only
// @shown preview: in the preview only (STYLE_RULE §4, where each card shows, checked by check-lego).
import { Bookmark, BookmarkPlus, Blend, FileInput, FolderInput, Layers, MoreHorizontal, Pencil, Plus, RefreshCw, TriangleAlert } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Hint } from '@/components/forms/field'
import { GroupRows } from '@/components/forms/group-rows'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { commands, type LookEntry, type PhotoId } from '@/ipc/bindings'
import { setHoverLook } from '@/lib/drag-state'
import { errorText } from '@/lib/errors'
import { SaveLookDialog } from './save-look-dialog'
import { useLookPicker } from './use-looks'

type PresetsCardProps = {
  photoId: PhotoId
  revision: number
}

/** A list this long gets its filter, as Lightroom's panel. */
const FILTER_FROM = 8

/** The groups open, kept while the app runs; a group not seen yet opens. */
const closed = new Set<string>()

/** What a dialog of the card is for. */
type Asking = { kind: 'save' } | { kind: 'rename' | 'move'; entry: LookEntry } | null

/** What the last import said, until the next action. */
type Report = { lines: string[]; problems: string[] } | null

/**
 * *Presets & LUTs*, as Lightroom's Presets panel on the left of its Develop view (owner,
 * 2026-11-07, 2026-11-08): every look of one's own, saved looks, LUTs and Lightroom presets,
 * in groups that fold, with a filter once the list is long; **+** saves the photo's look or
 * imports presets and LUTs; **⋯** and a right click rename, update with the photo's look,
 * move to a group or remove. Pro mode only.
 */
export function PresetsCard({ photoId, revision }: PresetsCardProps) {
  const { t } = useTranslation()
  const picker = useLookPicker(photoId, revision, undefined, true)
  const [asking, setAsking] = useState<Asking>(null)
  const [report, setReport] = useState<Report>(null)
  const [filter, setFilter] = useState('')
  const [menuFor, setMenuFor] = useState<string | null>(null)
  // The hover preview never outlives the card (a tab switched, the preview closed).
  useEffect(() => () => setHoverLook(null), [])
  const [, setClosedVersion] = useState(0)
  const own = picker.looks.filter((e) => e.kind === 'user')
  const chosen = picker.current === null ? undefined : picker.looks[picker.current]
  const shown = filter.trim() ? own.filter((e) => e.name.toLowerCase().includes(filter.trim().toLowerCase())) : own
  // Your looks first, then LUTs, then each preset group by name.
  const groups = new Map<string, LookEntry[]>()
  for (const e of shown) {
    const key = e.group ?? ''
    groups.set(key, [...(groups.get(key) ?? []), e])
  }
  const order = [...groups.keys()].sort((a, b) => (a === '' ? -1 : b === '' ? 1 : a === 'LUT' ? -1 : b === 'LUT' ? 1 : a.localeCompare(b)))
  const groupName = (key: string) => (key === '' ? t('look.yours') : key)
  const toggle = (key: string) => {
    if (closed.has(key)) closed.delete(key)
    else closed.add(key)
    setClosedVersion((v) => v + 1)
  }
  const fail = (r: { status: 'error'; error: { code: string; message: string; err?: string } }) => setReport({ lines: [], problems: [errorText(r.error)] })

  const importPresets = () =>
    void commands.importPresets().then((r) => {
      if (r.status !== 'ok') return fail(r)
      if (r.data === null) return
      const { imported, skipped, unapplied } = r.data
      picker.reload()
      const first = imported[0]
      if (first) picker.apply(first)
      setReport({
        lines: [
          t('look.importedPresets', { count: imported.length }),
          ...unapplied.map((u) => t('look.notAppliedLine', { setting: u.setting, count: u.count })),
        ],
        problems: skipped.map((s) => `${s.file}: ${t(`errors.${s.code}`, { defaultValue: s.reason })}`),
      })
    })
  const importLut = () =>
    void commands.importLut().then((r) => {
      if (r.status !== 'ok') return fail(r)
      setReport(null)
      if (r.data === null) return
      picker.reload()
      picker.apply(r.data)
    })

  const row = (entry: LookEntry) => (
    <div
      key={entry.id}
      className="group/row flex items-center gap-1"
      // As Lightroom shows a preset on the photo while it is hovered (owner, 2026-11-08).
      onMouseEnter={() => menuFor === null && setHoverLook(entry.id)}
      onMouseLeave={() => setHoverLook(null)}
      onContextMenu={(e) => {
        e.preventDefault()
        setMenuFor(entry.id)
      }}
    >
      <div className="min-w-0 flex-1" data-hint={entry.unapplied.length ? `${entry.name}\n${entry.unapplied.join('\n')}` : entry.name}>
        <ListRow icon={entry.source === 'lut' ? <Blend /> : <Bookmark />} chosen={chosen?.id === entry.id} onClick={() => picker.apply(entry)}>
          {entry.name}
        </ListRow>
      </div>
      <Popover open={menuFor === entry.id} onOpenChange={(open) => setMenuFor(open ? entry.id : null)}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            size="icon-sm"
            aria-label={t('look.actions')}
            data-hint={t('look.actions')}
            className="opacity-0 group-hover/row:opacity-100 focus-visible:opacity-100 aria-expanded:opacity-100"
          >
            <MoreHorizontal />
          </Button>
        </PopoverTrigger>
        <PopoverContent align="end" className="w-64 p-1">
          <ListRows>
            <ListRow icon={<Pencil />} onClick={() => (setMenuFor(null), setAsking({ kind: 'rename', entry }))}>
              {t('look.rename')}
            </ListRow>
            <ListRow
              icon={<RefreshCw />}
              onClick={() => {
                setMenuFor(null)
                void commands.getPhotoLook(photoId).then((c) => {
                  if (c.status === 'ok') void commands.updateLook(entry.id, c.data.look).then(() => picker.reload())
                })
              }}
            >
              {t('look.update')}
            </ListRow>
            <ListRow icon={<FolderInput />} onClick={() => (setMenuFor(null), setAsking({ kind: 'move', entry }))}>
              {t('look.move')}
            </ListRow>
            <RemoveButton wide small label={t('look.remove')} onClick={() => (setMenuFor(null), picker.remove(entry.id))} />
          </ListRows>
        </PopoverContent>
      </Popover>
    </div>
  )

  return (
    <SectionCard
      icon={<Layers />}
      title={t('look.presets')}
      action={
        <Popover>
          <PopoverTrigger asChild>
            <Button variant="outline" size="icon-sm" aria-label={t('look.add')} data-hint={t('look.add')}>
              <Plus />
            </Button>
          </PopoverTrigger>
          <PopoverContent align="end" className="w-64 p-1">
            <ListRows>
              <ListRow icon={<BookmarkPlus />} onClick={() => setAsking({ kind: 'save' })}>
                {t('look.saveAs')}
              </ListRow>
              <ListRow icon={<FileInput />} onClick={importPresets}>
                {t('look.importPresets')}
              </ListRow>
              <ListRow icon={<Blend />} onClick={importLut}>
                {t('look.importLut')}
              </ListRow>
            </ListRows>
          </PopoverContent>
        </Popover>
      }
    >
      {own.length >= FILTER_FROM && <Input value={filter} placeholder={t('look.filter')} aria-label={t('look.filter')} onChange={(e) => setFilter(e.target.value)} />}
      {own.length === 0 ? (
        <Hint>{t('look.presetsEmpty')}</Hint>
      ) : (
        <ListRows>
          {order.map((key) => (
            <GroupRows key={key} title={groupName(key)} count={groups.get(key)?.length ?? 0} open={!closed.has(key) || filter.trim() !== ''} onToggle={() => toggle(key)}>
              {(groups.get(key) ?? []).map(row)}
            </GroupRows>
          ))}
        </ListRows>
      )}
      {report && (
        <PanelNote
          danger={report.lines.length === 0}
          icon={<TriangleAlert />}
          action={
            <Button size="sm" onClick={() => setReport(null)}>
              {t('common.dismiss')}
            </Button>
          }
        >
          {[...report.lines, ...report.problems].map((line) => (
            <span key={line} className="block">
              {line}
            </span>
          ))}
        </PanelNote>
      )}
      <Hint>{t('look.importFormats')}</Hint>
      <SaveLookDialog
        open={asking !== null}
        title={asking?.kind === 'rename' ? t('look.rename') : asking?.kind === 'move' ? t('look.move') : undefined}
        body={asking?.kind === 'move' ? t('look.moveBody') : asking?.kind === 'rename' ? t('look.renameBody') : undefined}
        label={asking?.kind === 'move' ? t('look.group') : undefined}
        initial={asking?.kind === 'rename' ? asking.entry.name : asking?.kind === 'move' ? (asking.entry.group ?? '') : ''}
        okLabel={asking?.kind === 'move' ? t('look.moveOk') : undefined}
        onAnswer={(name) => {
          const now = asking
          setAsking(null)
          if (name === null || now === null) return
          if (now.kind === 'rename') void commands.renameLook(now.entry.id, name).then(() => picker.reload())
          else if (now.kind === 'move') void commands.moveLook(now.entry.id, name.trim() || null).then(() => picker.reload())
          else
            void commands.getPhotoLook(photoId).then((current) => {
              if (current.status !== 'ok') return
              void commands.saveLook(name, current.data.look).then((r) => {
                if (r.status !== 'ok') return fail(r)
                picker.reload()
                picker.apply(r.data)
              })
            })
        }}
      />
    </SectionCard>
  )
}
