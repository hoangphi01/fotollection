// SPDX-License-Identifier: GPL-3.0-only
import { FilterX, OctagonX } from 'lucide-react'
import { useState } from 'react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { CullFilterBar } from '@/features/culling/cull-filter-bar'
import { ALL_PHOTOS } from '@/features/culling/filter'
import { CULL_KEYS_HINT } from '@/features/culling/cull-keys'
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

export function App() {
  const [filter, setFilterState] = useState<CullFilter>(ALL_PHOTOS)
  const { photos, total, thumbVersions, lastImport, error, importFolder } = useLibrary(filter)
  const { settings, revision: renderRevision, update } = useRenderSettings()
  // Render and frame changes bump the same Rust counter; the newest one wins.
  const [frameRevision, setFrameRevision] = useState(0)
  const frame = useFrameConfig(setFrameRevision)
  const history = useHistory()
  const fonts = useFonts()
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
      />
      <main className="flex min-w-0 flex-1 flex-col">
        {error && (
          <Alert variant="destructive" className="m-3 w-auto">
            <OctagonX />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        <TaskProgressBar />
        {total === 0 ? (
          <LibraryEmptyState onOpenFolder={() => void importFolder()} />
        ) : (
          <>
            <div className="flex h-12 items-center gap-3 border-b px-4 text-sm">
              <span className="font-medium tabular-nums">
                {photos.length === total ? `${total} photos` : `${photos.length} / ${total} photos`}
              </span>
              <span className="truncate text-xs text-muted-foreground">
                Enter to preview · {CULL_KEYS_HINT}
              </span>
              <div className="ml-auto">
                <CullFilterBar filter={filter} onChange={setFilter} />
              </div>
            </div>
            {photos.length === 0 ? (
              <div className="flex flex-1 flex-col items-center justify-center gap-3 text-sm text-muted-foreground">
                <p>No photos match this filter.</p>
                <Button variant="outline" size="sm" onClick={() => setFilter(ALL_PHOTOS)}>
                  <FilterX data-icon="inline-start" /> Show all photos
                </Button>
              </div>
            ) : (
              <LibraryGrid
                photos={photos}
                thumbVersions={thumbVersions}
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
        onIndexChange={setSelected}
        onClose={() => setViewing(false)}
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
