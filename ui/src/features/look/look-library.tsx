// SPDX-License-Identifier: GPL-3.0-only
import { Blend, Bookmark, BookmarkPlus, FileInput, FolderInput, MoreHorizontal, Palette, Pencil, Plus, RefreshCw, TriangleAlert } from 'lucide-react'
import { useEffect, useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { Hint } from '@/components/forms/field'
import { GroupRows } from '@/components/forms/group-rows'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { PanelNote } from '@/components/layout/panel-note'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Popover, PopoverTrigger } from '@/components/ui/popover'
import { ScrollArea } from '@/components/ui/scroll-area'
import { commands, type LookEntry, type PhotoId } from '@/ipc/bindings'
import { setHoverLook } from '@/lib/drag-state'
import { errorText } from '@/lib/errors'
import { SaveLookDialog } from './save-look-dialog'
import { lookName } from './use-looks'
import { MenuContent } from '@/components/forms/menu-content'

/** A list this long gets its filter, as Lightroom's panel. */
const FILTER_FROM = 8

/** The built-in looks' group, first when they are listed. */
const BUILT_IN = '\u0000builtin'

/** The groups closed, kept while the app runs (the preview's and the gallery's alike). */
const closed = new Set<string>()

/** What a dialog of the list is for. */
type Asking = { kind: 'save' } | { kind: 'rename' | 'move'; entry: LookEntry } | null

/** What the last import said, until the next action. */
type Report = { lines: string[]; problems: string[] } | null

type LookLibraryOptions = {
  /** The library (`useLooks`), its reload after an import, a save or a rename, and removal. */
  looks: LookEntry[]
  reload: () => void
  remove: (id: string) => void
  /** The entry shown as chosen. */
  chosenId?: string
  /** A row clicked, or a look just imported or saved. */
  onPick: (entry: LookEntry) => void
  /** The photo whose look *Save as look* and *Update* take; without one they are not offered. */
  photoId?: PhotoId
  /** The built-in looks listed too, first (the gallery's Colour tab, FR-114). */
  builtIns?: boolean
  /** A hovered row shown on the photo through the GPU (the develop view's stage, FR-089). */
  hover?: boolean
  /** The rows scroll inside, about seven high, so what sits by the list stays in sight (the
   * gallery's Colour tab: its *Apply* above, as History's rows, FR-095). */
  scroll?: boolean
}

/**
 * The library of looks as Lightroom's Presets panel (FR-089, FR-114): one's own looks, LUTs
 * and Lightroom presets (and, with `builtIns`, the built-in looks), in groups that fold, with a
 * filter once the list is long; its **+** imports presets and LUTs (and saves the photo's look);
 * **⋯** and a right click rename, update, move to a group or remove. One block for the
 * preview's *Presets & LUTs* and the gallery's Colour tab (STYLE_RULE §4, the lego rule):
 * `action` goes in its card's header, `content` in its body.
 */
export function useLookLibrary({ looks, reload, remove, chosenId, onPick, photoId, builtIns = false, hover = false, scroll = false }: LookLibraryOptions): {
  action: ReactNode
  content: ReactNode
} {
  const { t } = useTranslation()
  const [asking, setAsking] = useState<Asking>(null)
  const [report, setReport] = useState<Report>(null)
  const [filter, setFilter] = useState('')
  const [menuFor, setMenuFor] = useState<string | null>(null)
  // The hover preview never outlives the list (a tab switched, the preview closed).
  useEffect(() => (hover ? () => setHoverLook(null) : undefined), [hover])
  const [, setClosedVersion] = useState(0)
  const listed = looks.filter((e) => builtIns || e.kind === 'user')
  const query = filter.trim().toLowerCase()
  const shown = query ? listed.filter((e) => lookName(e).toLowerCase().includes(query)) : listed
  // The built-in looks first, then yours, then LUTs, then each preset group by name.
  const groups = new Map<string, LookEntry[]>()
  for (const e of shown) {
    const key = e.kind === 'builtIn' ? BUILT_IN : (e.group ?? '')
    groups.set(key, [...(groups.get(key) ?? []), e])
  }
  const rank = (k: string) => (k === BUILT_IN ? 0 : k === '' ? 1 : k === 'LUT' ? 2 : 3)
  const order = [...groups.keys()].sort((a, b) => rank(a) - rank(b) || a.localeCompare(b))
  const groupName = (key: string) => (key === BUILT_IN ? t('look.builtins') : key === '' ? t('look.yours') : key)
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
      reload()
      const first = imported[0]
      if (first) onPick(first)
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
      reload()
      onPick(r.data)
    })

  const row = (entry: LookEntry) => {
    const own = entry.kind === 'user'
    return (
      <div
        key={entry.id}
        className="group/row flex items-center gap-1"
        // As Lightroom shows a preset on the photo while it is hovered (owner, 2026-11-08).
        onMouseEnter={() => hover && menuFor === null && setHoverLook(entry.id)}
        onMouseLeave={() => hover && setHoverLook(null)}
        onContextMenu={(e) => {
          e.preventDefault()
          if (own) setMenuFor(entry.id)
        }}
      >
        <div className="min-w-0 flex-1" data-hint={entry.unapplied.length ? `${lookName(entry)}\n${entry.unapplied.join('\n')}` : lookName(entry)}>
          <ListRow
            icon={entry.kind === 'builtIn' ? <Palette /> : entry.source === 'lut' ? <Blend /> : <Bookmark />}
            chosen={chosenId === entry.id}
            onClick={() => onPick(entry)}
          >
            {lookName(entry)}
          </ListRow>
        </div>
        {own && (
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
            <MenuContent align="end">
                <ListRow icon={<Pencil />} onClick={() => (setMenuFor(null), setAsking({ kind: 'rename', entry }))}>
                  {t('look.rename')}
                </ListRow>
                {photoId !== undefined && (
                  <ListRow
                    icon={<RefreshCw />}
                    onClick={() => {
                      setMenuFor(null)
                      void commands.getPhotoLook(photoId).then((c) => {
                        if (c.status === 'ok') void commands.updateLook(entry.id, c.data.look).then(() => reload())
                      })
                    }}
                  >
                    {t('look.update')}
                  </ListRow>
                )}
                <ListRow icon={<FolderInput />} onClick={() => (setMenuFor(null), setAsking({ kind: 'move', entry }))}>
                  {t('look.move')}
                </ListRow>
                <RemoveButton wide label={t('look.remove')} onClick={() => (setMenuFor(null), remove(entry.id))} />
              </MenuContent>
          </Popover>
        )}
      </div>
    )
  }

  const action = (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="outline" size="icon-sm" aria-label={t('look.add')} data-hint={t('look.add')}>
          <Plus />
        </Button>
      </PopoverTrigger>
      <MenuContent align="end">
          {photoId !== undefined && (
            <ListRow icon={<BookmarkPlus />} onClick={() => setAsking({ kind: 'save' })}>
              {t('look.saveAs')}
            </ListRow>
          )}
          <ListRow icon={<FileInput />} onClick={importPresets}>
            {t('look.importPresets')}
          </ListRow>
          <ListRow icon={<Blend />} onClick={importLut}>
            {t('look.importLut')}
          </ListRow>
        </MenuContent>
    </Popover>
  )

  const rows = (
    <ListRows>
      {order.map((key) => (
        <GroupRows key={key} title={groupName(key)} count={groups.get(key)?.length ?? 0} open={!closed.has(key) || query !== ''} onToggle={() => toggle(key)}>
          {(groups.get(key) ?? []).map(row)}
        </GroupRows>
      ))}
    </ListRows>
  )

  const content = (
    <>
      {listed.length >= FILTER_FROM && <Input value={filter} placeholder={t('look.filter')} aria-label={t('look.filter')} onChange={(e) => setFilter(e.target.value)} />}
      {listed.length === 0 ? (
        <Hint>{t('look.presetsEmpty')}</Hint>
      ) : scroll ? (
        <ScrollArea gutter className="max-h-72 [&>[data-slot=scroll-area-viewport]>div]:block!">{rows}</ScrollArea>
      ) : (
        rows
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
          if (now.kind === 'rename') void commands.renameLook(now.entry.id, name).then(() => reload())
          else if (now.kind === 'move') void commands.moveLook(now.entry.id, name.trim() || null).then(() => reload())
          else if (photoId !== undefined)
            void commands.getPhotoLook(photoId).then((current) => {
              if (current.status !== 'ok') return
              void commands.saveLook(name, current.data.look).then((r) => {
                if (r.status !== 'ok') return fail(r)
                reload()
                onPick(r.data)
              })
            })
        }}
      />
    </>
  )
  return { action, content }
}
