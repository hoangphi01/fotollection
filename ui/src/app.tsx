// SPDX-License-Identifier: GPL-3.0-only
import { FilterX, OctagonX, Sparkles, Square } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { GroupsView } from '@/features/best-shot/groups-view'
import { useBestShots } from '@/features/best-shot/use-best-shots'
import { CullFilterBar } from '@/features/culling/cull-filter-bar'
import { ALL_PHOTOS } from '@/features/culling/filter'
import { commands, events, type CullFilter, type OpenPhoto } from '@/ipc/bindings'
import { LibraryEmptyState } from '@/features/library/library-empty-state'
import { LibraryGrid } from '@/features/library/library-grid'
import { TaskProgressBar } from '@/features/library/task-progress-bar'
import { LibrarySidebar } from '@/features/library/library-sidebar'
import { SelectionBar } from '@/features/library/selection-bar'
import { DatePlaceDialog } from '@/features/places/date-place-dialog'
import { useLibrary } from '@/features/library/use-library'
import { SettingsPanel } from '@/features/settings/settings-panel'
import { useRenderSettings } from '@/features/settings/use-render-settings'
import { useLanguage } from '@/features/settings/use-language'
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
import { CameraSessionBanner } from '@/features/capture/camera-session-banner'
import { CaptureDialog } from '@/features/capture/capture-dialog'
import { ConnectCameraButton } from '@/features/capture/connect-camera-button'
import { DiscardSessionDialog, type SessionChoice } from '@/features/capture/discard-session-dialog'
import { useCapture } from '@/features/capture/use-capture'
import { SelectModeButton } from '@/features/library/select-mode-button'
import { useSelectMode } from '@/features/library/use-select-mode'
import { useAppMode } from '@/features/modes/use-app-mode'
import { useModeShortcuts } from '@/features/modes/use-mode-shortcuts'
import { CollagePanel } from '@/features/collage/collage-panel'
import { CollageView } from '@/features/collage/collage-view'
import { useCollage } from '@/features/collage/use-collage'
import { SimplePanel } from '@/features/simple/simple-panel'
import { SimpleView } from '@/features/simple/simple-view'

export function App() {
  const { t } = useTranslation()
  useStartupScreen()
  // The language applies in every mode, not only where its picker is (FR-020).
  useLanguage()
  // Simple mode shows one photo at a time and few settings (FR-048).
  const { mode, setMode } = useAppMode()
  useModeShortcuts(setMode)
  const simple = mode === 'simple'
  const collageMode = mode === 'collage'
  const [filter, setFilterState] = useState<CullFilter>(ALL_PHOTOS)
  // Simple and Collage follow the sidebar only (folders, basket, set aside); Pro's star and
  // flag filters stay in Pro, where they can be seen and cleared.
  const viewFilter: CullFilter = mode === 'professional' ? filter : { ...filter, flag: 'all', minRating: 0 }
  const { photos, total, folders, queueCounts, customStamps, thumbVersions, lastImport, error, importFolder, refresh, removeFolder } =
    useLibrary(viewFilter)
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
  // Tethered capture (FR-062, FR-063): the capture screen and the session tabs.
  // The camera went away by itself (unplugged, turned off): show the session's tab, as
  // closing the capture screen does (ADR-0028).
  const onCameraGone = useRef<() => void>(() => {})
  const capture = useCapture({ onGone: () => onCameraGone.current() })
  const [discarding, setDiscarding] = useState<number | null>(null)
  const [view, setView] = useState<'photos' | 'groups'>('photos')
  // The Relink dialog opens by itself after Open; "Later" hides it until the next project.
  const [relinkHiddenFor, setRelinkHiddenFor] = useState<number | null>(null)
  const revision = Math.max(renderRevision, frameRevision, history.revision)
  // Two photos on one print (FR-035): the sheets of the photos shown.
  const collage = useCollage(viewFilter, revision, collageMode)
  const [selected, setSelected] = useState(0)
  const [viewing, setViewing] = useState(false)
  // A rating or flag change can drop the current photo out of the filter; stay in range.
  const current = Math.max(0, Math.min(selected, photos.length - 1))
  // Select mode (FR-050), like the Select button on an iPhone: photos are ticked by id.
  const [hovered, setHovered] = useState<number | null>(null)
  // Esc closes Quick-preview first, and only then leaves select mode.
  const select = useSelectMode(photos, () => document.querySelector('[data-quick-preview]') !== null)
  const selectMode = select.active
  /** The ticked photos whose date and place are being set (FR-032). */
  const [datePlaceIds, setDatePlaceIds] = useState<number[] | null>(null)
  // Ticks follow the user across folder tabs (FR-056), so photos from several folders can be
  // gathered in one go; any other change of view starts again with none.
  const setFilter = (next: CullFilter, keepTicks = false) => {
    setFilterState(next)
    setSelected(0)
    if (!keepTicks) select.reset()
  }
  const quick = useQuickPreview(selectMode, photos.length, hovered, select.anchor ?? current, select.toggle)
  const quickPhoto = quick.index === null ? undefined : photos[quick.index]
  // Another mode starts clean: no select mode, no preview, no Quick-preview left over from
  // Pro (FR-048).
  const [shownMode, setShownMode] = useState(mode)
  if (shownMode !== mode) {
    setShownMode(mode)
    setViewing(false)
    select.leave()
  }

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
    // The photo opens in the preview, which is Pro's (FR-054, FR-048).
    setMode('professional')
    select.leave()
    setFilter({ ...ALL_PHOTOS, folders: [request.folder] })
    setPhotoRequest(request)
  })
  if (photoRequest && photos.length > 0 && photos.every((p) => p.folderId === photoRequest.folder)) {
    const index = photos.findIndex((p) => p.id === photoRequest.id)
    setPhotoRequest(null)
    if (index >= 0) open(index)
  }

  // Closing the capture screen lets the camera go and shows this session's tab (ADR-0028);
  // a shot clicked in its column opens in the preview, the same way as "Open with".
  const closeCapture = (shot?: OpenPhoto) => {
    const folder = shot?.folder ?? capture.sessionFolder
    void capture.disconnect()
    if (folder === null) return
    if (shot) setMode('professional')
    select.leave()
    setFilter({ ...ALL_PHOTOS, folders: [folder] })
    if (shot) setPhotoRequest(shot)
  }
  // Closing the app with session tabs not saved asks first (ADR-0028): Rust holds the close
  // and sends `quit-blocked`, and the answer ends with `quit_app`.
  const [quitting, setQuitting] = useState<number[] | null>(null)
  useEffect(() => {
    const unlisten = events.quitBlocked.listen((e) => setQuitting(e.payload.sessions))
    return () => void unlisten.then((off) => off())
  }, [])
  const chooseForQuit = async (choice: SessionChoice) => {
    const sessions = quitting
    setQuitting(null)
    if (sessions === null || choice === 'cancel') return
    await capture.disconnect()
    // Highest id first: deleting a tab renumbers the ones after it.
    for (const folder of [...sessions].sort((x, y) => y - x)) {
      const done = choice === 'save' ? await capture.saveSession(folder) : await capture.discardSession(folder)
      if (!done) return
    }
    await commands.quitApp()
  }

  useEffect(() => {
    onCameraGone.current = () => closeCapture()
  })
  const onlyFolder = filter.folders?.length === 1 ? filter.folders[0] : undefined
  const sessionTab =
    onlyFolder !== undefined && (filter.queue ?? 'gallery') === 'gallery' && capture.sessions.includes(onlyFolder)
      ? onlyFolder
      : null
  const discardSession = (folder: number) => {
    // Folder ids are renumbered when a tab goes: show every photo again.
    setFilter({ ...filter, folders: [], queue: 'gallery' })
    void capture.discardSession(folder)
  }
  const chooseForSession = (choice: SessionChoice) => {
    const folder = discarding
    setDiscarding(null)
    if (folder === null) return
    if (choice === 'save') void capture.saveSession(folder)
    else if (choice === 'discard') discardSession(folder)
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
          // A camera session's shots are only in the app's data: closing its tab asks first.
          if (capture.sessions.includes(id)) {
            setDiscarding(id)
            return
          }
          // Folder ids are renumbered after a removal: show every photo again.
          setFilter({ ...filter, folders: [], queue: 'gallery' })
          void removeFolder(id)
        }}
        project={project}
        onShowRelink={() => setRelinkHiddenFor(null)}
        cameraAction={<ConnectCameraButton capture={capture} />}
      />
      <main className="flex min-w-0 flex-1 flex-col">
        {(error ?? bestShots.error ?? (capture.phase === 'connected' ? null : capture.error)) && (
          <Alert variant="destructive" className="m-3 w-auto">
            <OctagonX />
            <AlertDescription>{error ?? bestShots.error ?? capture.error}</AlertDescription>
          </Alert>
        )}
        {sessionTab !== null && (
          <CameraSessionBanner
            count={folders.find((f) => f.id === sessionTab)?.count ?? 0}
            onSave={() => void capture.saveSession(sessionTab)}
            onDiscard={() => setDiscarding(sessionTab)}
          />
        )}
        <TaskProgressBar />
        {total === 0 ? (
          <LibraryEmptyState onOpenFolder={() => void importFolder()} />
        ) : collageMode ? (
          <CollageView
            collage={collage}
            photos={photos}
            settings={settings}
            onSettingsChange={(c) => void update(c)}
            exporter={exporter}
            filter={viewFilter}
            customStamps={customStamps}
            revision={revision}
            generation={project.document.generation}
          />
        ) : simple ? (
          <SimpleView
            photos={photos}
            index={current}
            onIndexChange={setSelected}
            view={filter.queue ?? 'gallery'}
            revision={revision}
            queueCounts={queueCounts}
          />
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
                  <SelectModeButton select={select} />
                  <CullFilterBar filter={filter} onChange={setFilter} />
                </div>
              )}
            </div>
            {view === 'photos' && selectMode && (
              <SelectionBar
                count={select.checked.size}
                elsewhere={select.checked.size - photos.filter((p) => select.checked.has(p.id)).length}
                view={filter.queue ?? 'gallery'}
                onMove={select.move}
                onDatePlace={() => setDatePlaceIds([...select.checked])}
                onSelectAll={select.tickAll}
                onClear={select.clear}
              />
            )}
            {datePlaceIds && <DatePlaceDialog ids={datePlaceIds} onClose={() => setDatePlaceIds(null)} />}
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
                checked={select.checked}
                onToggle={select.toggle}
                onToggleAll={select.tickAll}
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
          checked={select.checked.has(quickPhoto.id)}
          onClose={quick.close}
        />
      )}
      <PreviewDialog
        photos={photos}
        index={viewing && mode === 'professional' && photos.length > 0 ? current : null}
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
      <CaptureDialog
        capture={capture}
        settings={settings}
        thumbVersions={thumbVersions}
        generation={project.document.generation}
        onClose={() => closeCapture()}
        onOpenShot={(shot) => closeCapture({ id: shot.id, folder: shot.folder })}
      />
      <DiscardSessionDialog
        count={discarding === null ? null : (folders.find((f) => f.id === discarding)?.count ?? 0)}
        onChoose={chooseForSession}
      />
      <DiscardSessionDialog
        reason="quit"
        count={
          quitting === null
            ? null
            : folders.filter((f) => quitting.includes(f.id)).reduce((n, f) => n + f.count, 0)
        }
        onChoose={(choice) => void chooseForQuit(choice)}
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
      {collageMode ? (
        <CollagePanel
          settings={settings}
          onChange={(c) => void update(c)}
          exporter={exporter}
          filter={viewFilter}
          photoCount={photos.length}
          sheetCount={collage.sheets.length}
          cell={collage.cell}
          customStamps={customStamps}
          revision={revision}
        />
      ) : simple ? (
        <SimplePanel
          settings={settings}
          onChange={(c) => void update(c)}
          exporter={exporter}
          basketCount={queueCounts.print}
          customStamps={customStamps}
          revision={revision}
        />
      ) : (
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
        ticked={selectMode ? select.ordered : undefined}
      />
      )}
      </div>
    </div>
  )
}
