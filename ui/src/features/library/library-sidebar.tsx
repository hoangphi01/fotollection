// SPDX-License-Identifier: GPL-3.0-only
import {
  FileImage,
  FilePen,
  FilePlus,
  FileUp,
  Folder,
  FolderOpen,
  RefreshCw,
  Save,
  TriangleAlert,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { commands, type ImportSummary, type PhotoSummary } from '@/ipc/bindings'
import { ProjectName } from '@/features/project/project-name'
import type { ProjectApi } from '@/features/project/use-project'
import { errorText } from '@/lib/errors'

type LibrarySidebarProps = {
  photos: PhotoSummary[]
  lastImport: ImportSummary | null
  onOpenFolder: () => void
  /** Scan the folders again for new photos, keeping every edit. */
  onRefresh: () => void
  project: ProjectApi
  onShowRelink: () => void
}

export function LibrarySidebar({
  photos,
  lastImport,
  onOpenFolder,
  onRefresh,
  project,
  onShowRelink,
}: LibrarySidebarProps) {
  const { t } = useTranslation()
  // The app's own version, so people know which release they run (FR-021, no update check).
  const [version, setVersion] = useState<string | null>(null)
  useEffect(() => {
    void commands.appInfo().then((info) => setVersion(info.appVersion))
  }, [])
  const folders = useMemo(() => {
    const counts = new Map<string, number>()
    for (const p of photos) counts.set(p.folderName, (counts.get(p.folderName) ?? 0) + 1)
    return [...counts.entries()]
  }, [photos])

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
          <span className="truncate text-xs text-muted-foreground">
            {version ? t('app.versionByline', { version }) : t('app.byline')}
          </span>
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
                size="icon-xs"
                aria-label={t(`project.${key}`)}
                title={t(`project.${key}`)}
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
              title={t('project.edited')}
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
          {folders.length > 0 && (
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label={t('library.refresh')}
              title={t('library.refreshHint')}
              onClick={onRefresh}
            >
              <RefreshCw />
            </Button>
          )}
          <Button variant="ghost" size="icon-xs" aria-label={t('library.openFolder')} onClick={onOpenFolder}>
            <FolderOpen />
          </Button>
        </div>
      </div>
      <ScrollArea className="flex-1 px-2">
        {folders.length === 0 ? (
          <p className="px-2 py-1 text-xs text-muted-foreground">{t('library.noFolders')}</p>
        ) : (
          <ul className="space-y-0.5">
            {folders.map(([name, count]) => (
              <li
                key={name}
                className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-sidebar-accent"
              >
                <Folder className="size-4 text-muted-foreground" />
                <span className="truncate">{name}</span>
                <span className="ml-auto text-xs text-muted-foreground tabular-nums">{count}</span>
              </li>
            ))}
          </ul>
        )}
      </ScrollArea>
      {lastImport && lastImport.rejectedTotal > 0 && (
        <p
          className="border-t px-4 py-2 text-xs text-muted-foreground"
          title={lastImport.rejected
            .map((r) => `${r.fileName}: ${errorText({ code: r.code, message: r.code })}`)
            .join('\n')}
        >
          {t('library.skipped', { count: lastImport.rejectedTotal })}
        </p>
      )}
      <p className="px-4 pb-3 text-[11px] text-muted-foreground">{t('library.dropTip')}</p>
    </aside>
  )
}
