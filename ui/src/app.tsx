// SPDX-License-Identifier: GPL-3.0-only
import { FilterX, ListChecks, OctagonX, Sparkles, Square } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { GroupsView } from '@/features/best-shot/groups-view'
import { useBestShots } from '@/features/best-shot/use-best-shots'
import { CullFilterBar } from '@/features/culling/cull-filter-bar'
import { ALL_PHOTOS } from '@/features/culling/filter'
import { commands, type CullFilter, type OpenPhoto, type Queue } from '@/ipc/bindings'
import { LibraryEmptyState } from '@/features/library/library-empty-state'
import { LibraryGrid } from '@/features/library/library-grid'
import { TaskProgressBar } from '@/features/library/task-progress-bar'
import { LibrarySidebar } from '@/features/library/library-sidebar'
import { SelectionBar } from '@/features/library/selection-bar'
import { useLibrary } from '@/features/library/use-library'
import { SettingsPanel } from '@/features/settings/settings-panel'
import { useRenderSettings } from '@/features/settings/use-render-settings'
import { useFrameConfig } from '@/features/frame/use-frame-config'
import { useFonts } from '@/features/fonts/use-fonts'
import { useExport } from '@/features/export/use-export'
import { useHistory } from '@/features/history/use-history'
import { PreviewDialog } from '@/features/viewer/preview-dialog'
import { QuickPreview } from '@/features/viewer/quick-preview'
import { useQuickPreview } from '@/features/viewer/use-quick-preview'
import { TitleBar } from '@/features/window/title-bar'
import { RelinkDialog, UnsavedDialog } from '@/features/project/project-dialogs'
import { useProject } from '@/features/project/use-project'
import { useStartupScreen } from '@/features/startup/use-startup-screen'
import { useOpenWith } from '@/features/open-with/use-open-with'

export function App() {
  const { t } = useTranslation()
  useStartupScreen()
  const [filter, setFilterState] = useState<CullFilter>(ALL_PHOTOS)
  const { photos, total, folders, queueCounts, customStamps, thumbVersions, lastImport, error, importFolder, refresh, removeFolder } =
    useLibrary(filter)
  const { settings, revision: renderRevision, update, reload } = useRenderSettings()
  // Render and frame changes bump the same Rust counter; the newest one wins.
  const [frameRevision, setFrameRevision] = useState(0)
  const frame = useFrameConfig(setFrameRevision)
  const history = useHistory()
  const fonts = useFonts()
  // One export job and destination for the settings panel and the preview (FR-055).
  const exporter = useExport()
  const project = useProject()
  const bestShots = useBestShots()
  const [view, setView] = useState<'photos' | 'groups'>('photos')
  // The Relink dialog opens by itself after Open; "Later" hides it until the next project.
  const [relinkHiddenFor, setRelinkHiddenFor] = useState<number | null>(null)
  const revision = Math.max(renderRevision, frameRevision, history.revision)
  const [selected, setSelected] = useState(0)
  const [viewing, setViewing] = useState(false)
  // A rating or flag change can drop the current photo out of the filter; stay in range.
  const current = Math.max(0, Math.min(selected, photos.length - 1))
  // Select mode (FR-050), like the Select button on an iPhone: photos are ticked by id.
  const [selectMode, setSelectMode] = useState(false)
  const [checked, setChecked] = useState<ReadonlySet<number>>(new Set())
  const [anchor, setAnchor] = useState<number | null>(null)
  const [hovered, setHovered] = useState<number | null>(null)
  // Ticks follow the user across folder tabs (FR-056), so photos from several folders can be
  // gathered in one go; any other change of view starts again with none.
  const setFilter = (next: CullFilter, keepTicks = false) => {
    setFilterState(next)
    setSelected(0)
    if (!keepTicks) setChecked(new Set())
    setAnchor(null)
  }
  const tickAll = () => setChecked((prev) => new Set([...prev, ...photos.map((p) => p.id)]))
  const toggle = useCallback(
    (index: number, range = false) => {
      const id = photos[index]?.id
      if (id === undefined) return
      setChecked((prev) => {
        const next = new Set(prev)
        if (range && anchor !== null) {
          const [a, b] = [Math.min(anchor, index), Math.max(anchor, index)]
          for (const p of photos.slice(a, b + 1)) next.add(p.id)
        } else if (next.has(id)) next.delete(id)
        else next.add(id)
        return next
      })
      setAnchor(index)
    },
    [photos, anchor],
  )
  const leaveSelect = () => {
    setSelectMode(false)
    setChecked(new Set())
    setAnchor(null)
  }
  // Moves the ticked photos, in every folder tab, to the basket, aside, or back (FR-050); one
  // undoable step.
  const moveChecked = (queue: Queue | null) => {
    const ids = [...checked]
    if (ids.length > 0) void commands.setPhotoQueue(ids, queue)
    setChecked(new Set())
  }
  const quick = useQuickPreview(selectMode, photos.length, hovered, anchor ?? current, toggle)
  const quickPhoto = quick.index === null ? undefined : photos[quick.index]
  // Esc leaves select mode, unless a dialog or Quick-preview has it first.
  useEffect(() => {
    if (!selectMode) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || quick.index !== null || document.querySelector('[role="dialog"]')) return
      setSelectMode(false)
      setChecked(new Set())
      setAnchor(null)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [selectMode, quick.index])

  // Best-shot groups of the folders shown (FR-056): a group belongs to its first photo's folder.
  const scopedShots = filter.folders?.length
    ? {
        ...bestShots,
        groups: bestShots.groups.filter((g) => {
          const folder = bestShots.photosById.get(g.id)?.folderId
          return folder !== undefined && (filter.folders ?? []).includes(folder)
        }),
      }
    : bestShots

  const open = (index: number) => {
    setSelected(index)
    setViewing(true)
  }

  // "Open with" (FR-054): the photo's folder tab, then its preview once that folder's photos
  // are listed (an older list would give the wrong index).
  const [photoRequest, setPhotoRequest] = useState<OpenPhoto | null>(null)
  useOpenWith((request) => {
    setSelectMode(false)
    setFilter({ ...ALL_PHOTOS, folders: [request.folder] })
    setPhotoRequest(request)
  })
  if (photoRequest && photos.length > 0 && photos.every((p) => p.folderId === photoRequest.folder)) {
    const index = photos.findIndex((p) => p.id === photoRequest.id)
    setPhotoRequest(null)
    if (index >= 0) open(index)
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground select-none">
      <TitleBar name={project.document.name ?? t('project.untitled')} />
      <div className="flex min-h-0 flex-1">
      <LibrarySidebar
        folders={folders}
        selected={filter.folders ?? []}
        onSelect={(next) => setFilter({ ...filter, folders: next, queue: 'gallery' }, (filter.queue ?? 'gallery') === 'gallery')}
        queue={filter.queue ?? 'gallery'}
        onQueue={(queue) => setFilter({ ...filter, folders: [], queue })}
        queueCounts={queueCounts}
        lastImport={lastImport}
        onOpenFolder={() => void importFolder()}
        onRefresh={() => void refresh()}
        onRemoveFolder={(id) => {
          // Folder ids are renumbered after a removal: show every photo again.
          setFilter({ ...filter, folders: [], queue: 'gallery' })
          void removeFolder(id)
        }}
        project={project}
        onShowRelink={() => setRelinkHiddenFor(null)}
      />
      <main className="flex min-w-0 flex-1 flex-col">
        {(error ?? bestShots.error) && (
          <Alert variant="destructive" className="m-3 w-auto">
            <OctagonX />
            <AlertDescription>{error ?? bestShots.error}</AlertDescription>
          </Alert>
        )}
        <TaskProgressBar />
        {total === 0 ? (
          <LibraryEmptyState onOpenFolder={() => void importFolder()} />
        ) : (
          <>
            {/* Sized by the space between the sidebars, not the window: labels collapse to
                icons first, then the filters wrap to a second row. */}
            {/* Always one line (STYLE_RULE §4): filters are dropdowns, hints hide when narrow. */}
            <div className="@container flex min-h-[49px] flex-nowrap items-center gap-3 overflow-hidden border-b px-4 py-1.5 text-sm">
              <ToggleGroup
                type="single"
                variant="outline"
                size="sm"
                value={view}
                onValueChange={(v) => v && setView(v as 'photos' | 'groups')}
                aria-label={t('bestShot.view')}
              >
                <ToggleGroupItem value="photos" className="px-2.5 text-xs">
                  {t('bestShot.viewPhotos')}
                </ToggleGroupItem>
                <ToggleGroupItem value="groups" className="px-2.5 text-xs">
                  {t('bestShot.viewGroups', { count: scopedShots.groups.length })}
                </ToggleGroupItem>
              </ToggleGroup>
              <Tooltip>
                <TooltipTrigger asChild>
                  {bestShots.running ? (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={bestShots.cancel}
                      aria-label={t('bestShot.stop')}
                    >
                      <Square data-icon="inline-start" />
                      <span className="hidden @3xl:inline">{t('bestShot.stop')}</span>
                    </Button>
                  ) : (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setView('groups')
                        void bestShots.analyze()
                      }}
                      aria-label={t('bestShot.find')}
                    >
                      <Sparkles data-icon="inline-start" />
                      <span className="hidden @3xl:inline">{t('bestShot.find')}</span>
                    </Button>
                  )}
                </TooltipTrigger>
                <TooltipContent className="max-w-xs">{t('bestShot.findHint')}</TooltipContent>
              </Tooltip>
              {view === 'photos' && (
                <span className="shrink-0 font-medium whitespace-nowrap tabular-nums">
                  {photos.length === total
                    ? t('app.photos', { count: total })
                    : t('app.filtered', { shown: photos.length, count: total })}
                </span>
              )}
              {view === 'photos' && (
                <span className="hidden min-w-0 flex-1 truncate text-xs text-muted-foreground @6xl:inline">
                  {t('app.enterToPreview')} · {t('culling.keysHint')}
                </span>
              )}
              {view === 'photos' && (
                <div className="ml-auto flex items-center gap-2">
                  {/* Select mode (FR-050): tick photos for the basket or to set aside. */}
                  <Button
                    variant={selectMode ? 'default' : 'outline'}
                    size="sm"
                    aria-pressed={selectMode}
                    data-hint={t('select.hint')}
                    onClick={() => (selectMode ? leaveSelect() : setSelectMode(true))}
                  >
                    <ListChecks data-icon="inline-start" />
                    <span className="hidden @3xl:inline">{t(selectMode ? 'select.done' : 'select.start')}</span>
                  </Button>
                  <CullFilterBar filter={filter} onChange={setFilter} />
                </div>
              )}
            </div>
            {view === 'photos' && selectMode && (
              <SelectionBar
                count={checked.size}
                elsewhere={checked.size - photos.filter((p) => checked.has(p.id)).length}
                view={filter.queue ?? 'gallery'}
                onMove={moveChecked}
                onSelectAll={tickAll}
                onClear={() => setChecked(new Set())}
              />
            )}
            {view === 'groups' ? (
              <GroupsView
                bestShots={scopedShots}
                thumbVersions={thumbVersions}
                generation={project.document.generation}
                revision={revision}
              />
            ) : photos.length === 0 ? (
              <div className="flex flex-1 flex-col items-center justify-center gap-3 text-sm text-muted-foreground">
                <p>{t('app.noMatch')}</p>
                <Button variant="outline" size="sm" onClick={() => setFilter({ ...ALL_PHOTOS, folders: filter.folders })}>
                  <FilterX data-icon="inline-start" /> {t('app.showAll')}
                </Button>
              </div>
            ) : (
              <LibraryGrid
                photos={photos}
                thumbVersions={thumbVersions}
                generation={project.document.generation}
                selected={current}
                onSelect={setSelected}
                onOpen={open}
                selectMode={selectMode}
                checked={checked}
                onToggle={toggle}
                onToggleAll={tickAll}
                onHover={setHovered}
              />
            )}
          </>
        )}
      </main>
      {quickPhoto && (
        <QuickPreview
          photo={quickPhoto}
          revision={revision}
          thumbVersion={thumbVersions[quickPhoto.id] ?? 0}
          generation={project.document.generation}
          checked={checked.has(quickPhoto.id)}
          onClose={quick.close}
        />
      )}
      <PreviewDialog
        photos={photos}
        index={viewing && photos.length > 0 ? current : null}
        revision={revision}
        settings={settings}
        onSettingsChange={(c) => void update(c)}
        onSettingsReload={reload}
        frame={frame}
        fonts={fonts}
        exporter={exporter}
        filter={filter}
        onIndexChange={setSelected}
        onClose={() => setViewing(false)}
      />
      <UnsavedDialog
        name={project.document.name ?? t('project.untitled')}
        onChoose={project.question}
      />
      <RelinkDialog
        missing={project.document.missing}
        open={relinkHiddenFor !== project.document.generation}
        onLocate={(id) => void project.relink(id)}
        onClose={() => setRelinkHiddenFor(project.document.generation)}
      />
      <SettingsPanel
        settings={settings}
        onChange={(c) => void update(c)}
        frame={frame}
        fonts={fonts}
        exporter={exporter}
        photoCount={photos.length}
        folderCount={filter.folders?.length || folders.length}
        customStamps={customStamps}
        filter={filter}
        photo={photos[current]}
        revision={revision}
      />
      </div>
    </div>
  )
}
