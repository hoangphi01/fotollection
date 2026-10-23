// SPDX-License-Identifier: GPL-3.0-only
import { Folder, FolderOpen } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'

type LibrarySidebarProps = {
  folders: string[]
  onOpenFolder: () => void
}

function folderName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path
}

export function LibrarySidebar({ folders, onOpenFolder }: LibrarySidebarProps) {
  return (
    <aside className="flex w-60 shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground">
      <div className="flex h-12 items-center gap-2 px-4">
        <img src="/logo.png" alt="" className="size-7 rounded-md" />
        <span className="font-heading text-sm font-semibold tracking-tight">Fotollection</span>
      </div>
      <Separator />
      <div className="flex items-center justify-between px-4 pt-4 pb-2">
        <span className="text-xs font-medium text-muted-foreground uppercase">Library</span>
        <Button variant="ghost" size="icon-xs" aria-label="Open folder" onClick={onOpenFolder}>
          <FolderOpen />
        </Button>
      </div>
      <ScrollArea className="flex-1 px-2">
        {folders.length === 0 ? (
          <p className="px-2 py-1 text-xs text-muted-foreground">No folders yet</p>
        ) : (
          <ul className="space-y-0.5">
            {folders.map((path) => (
              <li
                key={path}
                title={path}
                className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-sidebar-accent"
              >
                <Folder className="size-4 text-muted-foreground" />
                <span className="truncate">{folderName(path)}</span>
              </li>
            ))}
          </ul>
        )}
      </ScrollArea>
    </aside>
  )
}
