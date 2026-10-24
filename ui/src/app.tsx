// SPDX-License-Identifier: GPL-3.0-only
import { FilterX, OctagonX, Sparkles, Square } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { GroupsView } from '@/features/best-shot/groups-view'
import { useBestShots } from '@/features/best-shot/use-best-shots'
import { CullFilterBar } from '@/features/culling/cull-filter-bar'
import { ALL_PHOTOS } from '@/features/culling/filter'
import type { CullFilter } from '@/ipc/bindings'
import { LibraryEmptyState } from '@/features/library/library-empty-state'
import { LibraryGrid } from '@/features/library/library-grid'
import { TaskProgressBar } from '@/features/library/task-progress-bar'
import { LibrarySidebar } from '@/features/library/library-sidebar'
import { useLibrary } from '@/features/library/use-library'
import { SettingsPanel } from '@/features/settings/settings-panel'
import { useRenderSettings } from '@/features/settings/use-render-settings'
import { useFrameConfig } from '@/features/frame/use-frame-config'
import { useFonts } from '@/features/fonts/use-fonts'
import { useHistory } from '@/features/history/use-history'
import { PreviewDialog } from '@/features/viewer/preview-dialog'
import { RelinkDialog, UnsavedDialog } from '@/features/project/project-dialogs'
import { useProject } from '@/features/project/use-project'

export function App() {
  const { t } = useTranslation()
  const [filter, setFilterState] = useState<CullFilter>(ALL_PHOTOS)
  const { photos, total, thumbVersions, lastImport, error, importFolder } = useLibrary(filter)
  const { settings, revision: renderRevision, update } = useRenderSettings()
  // Render and frame changes bump the same Rust counter; the newest one wins.
  const [frameRevision, setFrameRevision] = useState(0)
  const frame = useFrameConfig(setFrameRevision)
  const history = useHistory()
  const fonts = useFonts()
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
  const setFilter = (next: CullFilter) => {
    setFilterState(next)
    setSelected(0)
  }

  const open = (index: number) => {
    setSelected(index)
    setViewing(true)
  }

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground select-none">
      <LibrarySidebar
        photos={photos}
        lastImport={lastImport}
        onOpenFolder={() => void importFolder()}
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
            <div className="flex h-12 items-center gap-3 border-b px-4 text-sm">
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
                  {t('bestShot.viewGroups', { count: bestShots.groups.length })}
                </ToggleGroupItem>
              </ToggleGroup>
              <Tooltip>
                <TooltipTrigger asChild>
                  {bestShots.running ? (
                    <Button variant="outline" size="sm" onClick={bestShots.cancel}>
                      <Square data-icon="inline-start" /> {t('bestShot.stop')}
                    </Button>
                  ) : (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setView('groups')
                        void bestShots.analyze()
                      }}
                    >
                      <Sparkles data-icon="inline-start" /> {t('bestShot.find')}
                    </Button>
                  )}
                </TooltipTrigger>
                <TooltipContent className="max-w-xs">{t('bestShot.findHint')}</TooltipContent>
              </Tooltip>
              {view === 'photos' && (
                <span className="font-medium tabular-nums">
                  {photos.length === total
                    ? t('app.photos', { count: total })
                    : t('app.filtered', { shown: photos.length, count: total })}
                </span>
              )}
              {view === 'photos' && (
                <span className="truncate text-xs text-muted-foreground">
                  {t('app.enterToPreview')} · {t('culling.keysHint')}
                </span>
              )}
              {view === 'photos' && (
                <div className="ml-auto">
                  <CullFilterBar filter={filter} onChange={setFilter} />
                </div>
              )}
            </div>
            {view === 'groups' ? (
              <GroupsView
                bestShots={bestShots}
                thumbVersions={thumbVersions}
                generation={project.document.generation}
                revision={revision}
              />
            ) : photos.length === 0 ? (
              <div className="flex flex-1 flex-col items-center justify-center gap-3 text-sm text-muted-foreground">
                <p>{t('app.noMatch')}</p>
                <Button variant="outline" size="sm" onClick={() => setFilter(ALL_PHOTOS)}>
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
              />
            )}
          </>
        )}
      </main>
      <PreviewDialog
        photos={photos}
        index={viewing && photos.length > 0 ? current : null}
        revision={revision}
        settings={settings}
        onSettingsChange={(c) => void update(c)}
        frame={frame}
        fonts={fonts}
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
        photoCount={photos.length}
        filter={filter}
      />
    </div>
  )
}
