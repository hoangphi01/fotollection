// SPDX-License-Identifier: GPL-3.0-only
import { FolderOpen, Images } from 'lucide-react'
import { Button } from '@/components/ui/button'

type LibraryEmptyStateProps = {
  onOpenFolder: () => void
}

export function LibraryEmptyState({ onOpenFolder }: LibraryEmptyStateProps) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <div className="flex size-14 items-center justify-center rounded-2xl bg-muted">
        <Images className="size-7 text-muted-foreground" />
      </div>
      <div className="space-y-1">
        <h1 className="font-heading text-lg font-semibold">Start with a folder of photos</h1>
        <p className="max-w-sm text-sm text-muted-foreground">
          JPEG, HEIC and PNG. Everything stays on this computer.
        </p>
      </div>
      <Button onClick={onOpenFolder}>
        <FolderOpen data-icon="inline-start" />
        Open folder
      </Button>
    </div>
  )
}
