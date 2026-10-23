// SPDX-License-Identifier: GPL-3.0-only
import { Folder, FolderOpen } from 'lucide-react'
import { useMemo } from 'react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import type { ImportSummary, PhotoSummary } from '@/ipc/bindings'

type LibrarySidebarProps = {
  photos: PhotoSummary[]
  lastImport: ImportSummary | null
  onOpenFolder: () => void
}

export function LibrarySidebar({ photos, lastImport, onOpenFolder }: LibrarySidebarProps) {
  const folders = useMemo(() => {
    const counts = new Map<string, number>()
    for (const p of photos) counts.set(p.folderName, (counts.get(p.folderName) ?? 0) + 1)
    return [...counts.entries()]
  }, [photos])

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground">
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
          title={lastImport.rejected.map((r) => `${r.fileName} (${r.code})`).join('\n')}
        >
          {lastImport.rejectedTotal} file{lastImport.rejectedTotal > 1 ? 's' : ''} skipped
          (HEIC/PNG/RAW coming soon)
        </p>
      )}
      <p className="px-4 pb-3 text-[11px] text-muted-foreground">Tip: drop a folder anywhere</p>
    </aside>
  )
}
