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
  TriangleAlert,
  X,
} from 'lucide-react'
import { useState, type MouseEvent, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import type { FolderSummary, ImportSummary, QueueView } from '@/ipc/bindings'
import { ThemeSwitch } from '@/features/appearance/theme-switch'
import { ProjectName } from '@/features/project/project-name'
import type { ProjectApi } from '@/features/project/use-project'
import { errorText } from '@/lib/errors'
import { cn } from '@/lib/utils'

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
  const rowClass = (active: boolean) =>
    cn(
      'flex w-full min-w-0 items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-sidebar-accent',
      active && 'bg-sidebar-accent font-medium text-sidebar-accent-foreground',
    )

  return (
    <aside className="flex w-72 shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground">
      <div className="flex h-12 items-center gap-2 px-4">
        {/* Simplified mark (no wordmark) pre-rendered per density, so WebKit never downscales */}
        <img
          src="/brand/mark-28.png"
          srcSet="/brand/mark-28.png 1x, /brand/mark-56.png 2x, /brand/mark-84.png 3x"
          width={28}
          height={28}
          alt=""
          className="rounded-md"
        />
        <div className="flex min-w-0 items-baseline gap-1.5">
          <span className="font-heading text-sm font-semibold tracking-tight">Fotollection</span>
          <span className="truncate text-xs text-muted-foreground">{t('app.byline')}</span>
        </div>
      </div>
      <Separator />
      {/* The .fotollection document (FR-023). */}
      <div className="space-y-1.5 px-4 pt-4 pb-3">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-muted-foreground uppercase">
            {t('project.title')}
          </span>
          <div className="flex">
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
          </div>
        </div>
        <p className="flex min-w-0 items-center gap-2 text-sm">
          <FileImage className="size-4 shrink-0 text-muted-foreground" />
          <ProjectName name={project.document.name} onRename={project.rename} />
          {project.document.edited && (
            <span
              className="size-1.5 shrink-0 rounded-full bg-amber-500"
              data-hint={t('project.edited')}
              role="img"
              aria-label={t('project.edited')}
            />
          )}
        </p>
        {project.document.missing.length > 0 && (
          <button
            type="button"
            onClick={onShowRelink}
            className="flex items-center gap-1.5 text-xs text-amber-600 hover:underline dark:text-amber-400"
          >
            <TriangleAlert className="size-3.5" />
            {t('project.missing', { count: project.document.missing.length })}
          </button>
        )}
        {project.error && <p className="text-xs text-destructive">{project.error}</p>}
      </div>
      <Separator />
      <div className="flex items-center justify-between px-4 pt-4 pb-2">
        <span className="text-xs font-medium text-muted-foreground uppercase">
          {t('library.title')}
        </span>
        <div className="flex">
          {folders.length > 1 && (
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('library.select')}
              aria-pressed={selecting}
              data-hint={t('library.selectHint')}
              className={cn(selecting && 'bg-sidebar-accent text-primary')}
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
        </div>
      </div>
      <ScrollArea className="flex-1 px-2">
        {folders.length === 0 ? (
          <p className="px-2 py-1 text-xs text-muted-foreground">{t('library.noFolders')}</p>
        ) : (
          <ul className="space-y-0.5">
            <li>
              <button
                type="button"
                className={rowClass(inGallery && selected.length === 0)}
                aria-pressed={inGallery && selected.length === 0}
                data-hint={t('library.allHint')}
                onClick={() => onSelect([])}
              >
                <Images className={cn('size-4 shrink-0', inGallery && selected.length === 0 ? 'text-primary' : 'text-muted-foreground')} />
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
                  <Icon className={cn('size-4 shrink-0', queue === view ? 'text-primary' : 'text-muted-foreground')} />
                  <span className="truncate">{t(label)}</span>
                  <span className="ml-auto text-xs text-muted-foreground tabular-nums">{count}</span>
                </button>
              </li>
            ))}
            {folders.map(({ id, name, count }) => {
              const active = inGallery && selected.includes(id)
              return (
                <li key={id} className="group relative flex items-center">
                  <button
                    type="button"
                    className={rowClass(active)}
                    aria-pressed={active}
                    data-hint={selecting ? undefined : t('library.folderHint')}
                    onClick={(e) => pick(e, id)}
                  >
                    {selecting ? (
                      <Checkbox checked={active} tabIndex={-1} className="pointer-events-none" aria-hidden />
                    ) : (
                      <Folder className={cn('size-4 shrink-0', active ? 'text-primary' : 'text-muted-foreground')} />
                    )}
                    <span className="truncate">{name}</span>
                    <span className="ml-auto text-xs text-muted-foreground tabular-nums group-focus-within:invisible group-hover:invisible">
                      {count}
                    </span>
                  </button>
                  {/* Like closing a browser tab: shown on hover, the files stay on disk. */}
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    aria-label={t('library.removeFolder', { name })}
                    data-hint={t('library.removeFolderHint')}
                    onClick={() => onRemoveFolder(id)}
                    className="absolute right-1.5 hidden size-5 rounded-full group-focus-within:inline-flex group-hover:inline-flex"
                  >
                    <X className="size-3.5" />
                  </Button>
                </li>
              )
            })}
          </ul>
        )}
      </ScrollArea>
      {lastImport && lastImport.fromXmp > 0 && (
        <p className="border-t px-4 py-2 text-xs text-muted-foreground">
          {t('library.fromXmp', { count: lastImport.fromXmp })}
        </p>
      )}
      {lastImport && lastImport.rejectedTotal > 0 && (
        <p
          className="border-t px-4 py-2 text-xs text-muted-foreground"
          data-hint={lastImport.rejected
            .map((r) => `${r.fileName}: ${errorText({ code: r.code, message: r.code })}`)
            .join('\n')}
        >
          {t('library.skipped', { count: lastImport.rejectedTotal })}
        </p>
      )}
      <div className="flex items-center justify-between gap-2 px-4 pb-3">
        <p className="text-[11px] text-muted-foreground">{t('library.dropTip')}</p>
        <ThemeSwitch />
      </div>
    </aside>
  )
}
