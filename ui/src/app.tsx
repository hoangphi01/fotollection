// SPDX-License-Identifier: GPL-3.0-only
import { LibraryEmptyState } from '@/features/library/library-empty-state'
import { LibrarySidebar } from '@/features/library/library-sidebar'
import { useOpenFolder } from '@/features/library/use-open-folder'
import { SettingsPanel } from '@/features/settings/settings-panel'

export function App() {
  const { folders, openFolder } = useOpenFolder()

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground select-none">
      <LibrarySidebar folders={folders} onOpenFolder={openFolder} />
      <main className="flex min-w-0 flex-1 flex-col">
        <LibraryEmptyState onOpenFolder={openFolder} />
      </main>
      <SettingsPanel />
    </div>
  )
}
