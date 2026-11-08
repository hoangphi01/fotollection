// SPDX-License-Identifier: GPL-3.0-only
import {
  FileImage,
  FilePen,
  FilePlus,
  FileUp,
  Folder,
  FolderOpen,
  Archive,
  Images,
  ListChecks,
  ShoppingBasket,
  RefreshCw,
  Save,
  Settings,
  TriangleAlert,
} from 'lucide-react'
import { useState, type MouseEvent, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { ButtonGroup } from '@/components/forms/button-group'
import { commands, type FolderSummary, type ImportSummary, type QueueView } from '@/ipc/bindings'
import { ThemeSwitch } from '@/features/appearance/theme-switch'
import { ProjectName } from '@/features/project/project-name'
import type { ProjectApi } from '@/features/project/use-project'
import { errorText } from '@/lib/errors'
import { cn } from '@/lib/utils'
import { ModeSwitch } from '@/features/modes/mode-switch'
import { useFolderDrag } from './use-folder-drag'
import { showSettings } from '@/features/settings-window/use-settings-window'
import { RemoveButton } from '@/components/forms/danger-button'
import { FieldLabel } from '@/components/forms/field'
import { PanelNote } from '@/components/layout/panel-note'
import { CHOSEN_ROW, ROW_HOVER, ROW_ICON, ROW_LAYOUT } from '@/components/forms/list-row'

type LibrarySidebarProps = {
  /** Every source folder with its own count (FR-056). */
  folders: FolderSummary[]
  /** Folders shown, like browser tabs; empty means all photos. */
  selected: number[]
  onSelect: (folders: number[]) => void
  /** The gallery, the print basket or the photos set aside (FR-050). */
  queue: QueueView
  onQueue: (queue: QueueView) => void
  queueCounts: { print: number; aside: number }
  lastImport: ImportSummary | null
  onOpenFolder: () => void
  /** Scan the folders again for new photos, keeping every edit. */
  onRefresh: () => void
  /** Take a folder out of the library, like closing a tab; files stay on disk. */
  onRemoveFolder: (folder: number) => void
  project: ProjectApi
  onShowRelink: () => void
  /** Connect camera (FR-063), shown before Open folder. */
  cameraAction?: ReactNode
}

export function LibrarySidebar({
  folders,
  selected,
  onSelect,
  queue,
  onQueue,
  queueCounts,
  lastImport,
  onOpenFolder,
  onRefresh,
  onRemoveFolder,
  project,
  onShowRelink,
  cameraAction,
}: LibrarySidebarProps) {
  const { t } = useTranslation()
  const total = folders.reduce((sum, f) => sum + f.count, 0)
  // Select mode: a tick box per folder, for showing or exporting several at once.
  const [selecting, setSelecting] = useState(false)
  // A click shows one folder, like switching tabs; in select mode, or with Ctrl/⌘-click,
  // it adds or removes one.
  const pick = (event: MouseEvent, id: number) => {
    if (!(selecting || event.metaKey || event.ctrlKey)) return onSelect([id])
    const next = selected.includes(id) ? selected.filter((f) => f !== id) : [...selected, id]
    onSelect(next)
  }
  const inGallery = queue === 'gallery'
  // Folder tabs in the order the user drags them, as Chrome's (FR-107); Rust keeps the order.
  const folderById = new Map(folders.map((f) => [f.id, f]))
  const folderDrag = useFolderDrag(
    folders.map((f) => f.id),
    (id, to) => void commands.moveFolder(id, to),
  )
  const rowClass = (active: boolean) =>
    cn(
      'flex h-8 w-full min-w-0 items-center rounded-md text-left',
      ROW_LAYOUT,
      ROW_HOVER,
      ROW_ICON(active),
      // The chosen row: an accent outline inside, as Sentry's selected project (FR-099).
      active && CHOSEN_ROW,
    )

  return (
    <aside className="flex w-panel shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground">
      <div className="flex h-header shrink-0 items-center gap-2 px-4">
        {/* Simplified mark (no wordmark) pre-rendered per density, so WebKit never downscales */}
        {/* The mark, as tall as the gear (32 px); a click opens fotollection.org in the browser. */}
        <button
          type="button"
          className="shrink-0 rounded-md outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
          aria-label={t('app.website')}
          data-hint={t('app.website')}
          onClick={() => void commands.openWebsite()}
        >
          <img
            src="/brand/mark-32.png"
            srcSet="/brand/mark-32.png 1x, /brand/mark-64.png 2x, /brand/mark-96.png 3x"
            width={32}
            height={32}
            alt=""
            className="rounded-md"
          />
        </button>
        {/* The name above, the author a size smaller below it (owner, 2026-11-06). */}
        <div className="flex min-w-0 flex-col">
          <span className="font-heading text-sm leading-tight font-semibold tracking-tight">Fotollection</span>
          <span className="truncate text-2xs leading-snug text-muted-foreground">{t('app.byline')}</span>
        </div>
        {/* Every setting of this computer in one window (FR-086). */}
        <Button
          variant="outline"
          size="icon-sm"
          className="ml-auto shrink-0"
          aria-label={t('settingsWindow.title')}
          data-hint={t('settingsWindow.open')}
          onClick={() => showSettings('general')}
        >
          <Settings />
        </Button>
      </div>
      <Separator />
      {/* Every block 12 px in from the edge and 12 px apart, so the mode switch, the button
          groups and the chosen row's outline line up (STYLE_RULE §4, three spacings). */}
      <div className="px-3 pt-3">
        <ModeSwitch />
      </div>
      {/* The .fotollection document (FR-023). */}
      <div className="space-y-1.5 px-3 pt-3 pb-3">
        <div className="flex items-center justify-between">
          <FieldLabel>{t('project.title')}</FieldLabel>
          <ButtonGroup ariaLabel={t('project.title')}>
            {(
              [
                ['new', FilePlus, project.newProject],
                ['open', FileUp, project.open],
                ['save', Save, project.save],
                ['saveAs', FilePen, project.saveAs],
              ] as const
            ).map(([key, Icon, action]) => (
              <Button
                key={key}
                variant="ghost"
                size="icon-sm"
                aria-label={t(`project.${key}`)}
                data-hint={t(`project.${key}`)}
                onClick={() => void action()}
              >
                <Icon />
              </Button>
            ))}
          </ButtonGroup>
        </div>
        <p className="flex min-w-0 items-center gap-2 text-sm">
          <FileImage className="size-4 shrink-0 text-muted-foreground" />
          <ProjectName name={project.document.name} onRename={project.rename} />
          {project.document.edited && (
            <span
              className="size-1.5 shrink-0 rounded-full bg-warning"
              data-hint={t('project.edited')}
              role="img"
              aria-label={t('project.edited')}
            />
          )}
        </p>
        {/* A note with its action, never a link-styled line (STYLE_RULE §4). */}
        {project.document.missing.length > 0 && (
          <PanelNote
            icon={<TriangleAlert />}
            action={
              <Button size="sm" onClick={onShowRelink}>
                {t('project.locate')}
              </Button>
            }
          >
            {t('project.missing', { count: project.document.missing.length })}
          </PanelNote>
        )}
        {project.error && (
          <PanelNote danger icon={<TriangleAlert />}>
            {project.error}
          </PanelNote>
        )}
      </div>
      <Separator />
      <div className="flex items-center justify-between px-3 pt-3 pb-2">
        <FieldLabel>{t('library.title')}</FieldLabel>
        <ButtonGroup ariaLabel={t('library.title')}>
          {folders.length > 1 && (
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('library.select')}
              aria-pressed={selecting}
              data-hint={t('library.selectHint')}
              className={cn(selecting && 'bg-pressed text-primary-foreground pressed-in [--face:var(--primary)]')}
              onClick={() => setSelecting((on) => !on)}
            >
              <ListChecks />
            </Button>
          )}
          {folders.length > 0 && (
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('library.refresh')}
              data-hint={t('library.refreshHint')}
              onClick={onRefresh}
            >
              <RefreshCw />
            </Button>
          )}
          {cameraAction}
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={t('library.openFolder')}
            data-hint={t('library.openFolder')}
            onClick={onOpenFolder}
          >
            <FolderOpen />
          </Button>
        </ButtonGroup>
      </div>
      <ScrollArea className="flex-1 px-3">
        {folders.length === 0 ? (
          <p className="px-2 py-1 text-xs text-muted-foreground">{t('library.noFolders')}</p>
        ) : (
          <ul className="flex flex-col gap-1">
            <li>
              <button
                type="button"
                className={rowClass(inGallery && selected.length === 0)}
                aria-pressed={inGallery && selected.length === 0}
                data-hint={t('library.allHint')}
                onClick={() => onSelect([])}
              >
                <Images />
                <span className="truncate">{t('library.allPhotos')}</span>
                <span className="ml-auto text-xs text-muted-foreground tabular-nums">{total - queueCounts.aside}</span>
              </button>
            </li>
            {/* The two holding areas of the print workflow (FR-050): what will be printed, and
                what was set aside, out of the gallery but never deleted. */}
            {(
              [
                ['print', ShoppingBasket, 'library.basket', 'library.basketHint', queueCounts.print],
                ['aside', Archive, 'library.aside', 'library.asideHint', queueCounts.aside],
              ] as const
            ).map(([view, Icon, label, hint, count]) => (
              <li key={view}>
                <button
                  type="button"
                  className={rowClass(queue === view)}
                  aria-pressed={queue === view}
                  data-hint={t(hint)}
                  onClick={() => onQueue(view)}
                >
                  <Icon />
                  <span className="truncate">{t(label)}</span>
                  <span className="ml-auto text-xs text-muted-foreground tabular-nums">{count}</span>
                </button>
              </li>
            ))}
            {folderDrag.order.map((id) => {
              const folder = folderById.get(id)
              if (!folder) return null
              const { name, count } = folder
              const active = inGallery && selected.includes(id)
              const lifted = folderDrag.dragging === id
              return (
                <li
                  key={id}
                  {...folderDrag.rowProps(id)}
                  className={cn(
                    'group relative flex items-center rounded-md motion-move',
                    lifted && 'bg-sidebar lifted transition-none',
                  )}
                >
                  <button
                    type="button"
                    className={cn(rowClass(active), folderDrag.dragging === null ? 'cursor-grab' : 'cursor-grabbing')}
                    aria-pressed={active}
                    data-hint={selecting || folderDrag.dragging !== null ? undefined : t('library.folderHint')}
                    {...folderDrag.handleProps(id)}
                    onClick={(e) => {
                      if (!folderDrag.takeSuppressedClick()) pick(e, id)
                    }}
                  >
                    {selecting ? (
                      <Checkbox checked={active} tabIndex={-1} className="pointer-events-none" aria-hidden />
                    ) : (
                      <Folder />
                    )}
                    <span className="truncate">{name}</span>
                    <span className="ml-auto text-xs text-muted-foreground tabular-nums group-focus-within:invisible group-hover:invisible">
                      {count}
                    </span>
                  </button>
                  {/* Like closing a browser tab: shown on hover, the files stay on disk. */}
                  {folderDrag.dragging === null && (
                    <RemoveButton
                      iconOnly
                      small
                      round
                      label={t('library.removeFolder', { name })}
                      hint={t('library.removeFolderHint')}
                      onClick={() => onRemoveFolder(id)}
                      className="absolute right-1.5 hidden group-focus-within:inline-flex group-hover:inline-flex"
                    />
                  )}
                </li>
              )
            })}
          </ul>
        )}
      </ScrollArea>
      {lastImport && lastImport.fromXmp > 0 && (
        <p className="border-t px-3 py-2 text-xs text-muted-foreground">{t('library.fromXmp', { count: lastImport.fromXmp })}</p>
      )}
      {lastImport && lastImport.rejectedTotal > 0 && (
        <p
          className="border-t px-3 py-2 text-xs text-muted-foreground"
          data-hint={lastImport.rejected
            .map((r) => `${r.fileName}: ${errorText({ code: r.code, message: r.code, err: r.err })}`)
            .join('\n')}
        >
          {t('library.skipped', { count: lastImport.rejectedTotal })}
        </p>
      )}
      <div className="flex items-center justify-between gap-2 px-3 py-3">
        <p className="text-xs text-muted-foreground">{t('library.dropTip')}</p>
        <ThemeSwitch />
      </div>
    </aside>
  )
}
