// SPDX-License-Identifier: GPL-3.0-only
import { OctagonX } from 'lucide-react'
import { useState } from 'react'
import { Alert, AlertDescription } from '@/components/ui/alert'
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
  const { photos, thumbVersions, lastImport, error, importFolder } = useLibrary()
  const { settings, revision: renderRevision, update } = useRenderSettings()
  // Render and frame changes bump the same Rust counter; the newest one wins.
  const [frameRevision, setFrameRevision] = useState(0)
  const frame = useFrameConfig(setFrameRevision)
  const history = useHistory()
  const fonts = useFonts()
  const revision = Math.max(renderRevision, frameRevision, history.revision)
  const [selected, setSelected] = useState(0)
  const [viewing, setViewing] = useState(false)

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
        {photos.length === 0 ? (
          <LibraryEmptyState onOpenFolder={() => void importFolder()} />
        ) : (
          <>
            <div className="flex h-12 items-center border-b px-4 text-sm">
              <span className="font-medium">{photos.length} photos</span>
              <span className="ml-2 text-xs text-muted-foreground">
                Double-click, Enter or Space to open the print preview
              </span>
            </div>
            <LibraryGrid
              photos={photos}
              thumbVersions={thumbVersions}
              selected={selected}
              onSelect={setSelected}
              onOpen={open}
            />
          </>
        )}
      </main>
      <PreviewDialog
        photos={photos}
        index={viewing ? selected : null}
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
      />
    </div>
  )
}
