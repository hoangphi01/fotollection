// SPDX-License-Identifier: GPL-3.0-only
import { ChevronLeft, ChevronRight, LayoutGrid, LoaderCircle } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import type { PhotoSummary } from '@/ipc/bindings'
import { previewUrl } from '@/lib/photo-url'

type PhotoViewerProps = {
  photos: PhotoSummary[]
  index: number
  revision: number
  onIndexChange: (index: number) => void
  onClose: () => void
}

/** Shows the Rust-rendered preview: the exact crop and stamp that will be exported (FR-007). */
export function PhotoViewer({ photos, index, revision, onIndexChange, onClose }: PhotoViewerProps) {
  const photo = photos[index]
  const src = photo ? previewUrl(photo.id, revision) : ''
  const [loadedSrc, setLoadedSrc] = useState('')

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
      else if (event.key === 'ArrowRight') onIndexChange(Math.min(photos.length - 1, index + 1))
      else if (event.key === 'ArrowLeft') onIndexChange(Math.max(0, index - 1))
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [index, photos.length, onClose, onIndexChange])

  if (!photo) return null

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex h-12 items-center gap-2 border-b px-3">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon-sm" aria-label="Back to grid" onClick={onClose}>
              <LayoutGrid />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Back to grid (Esc)</TooltipContent>
        </Tooltip>
        <span className="truncate text-sm font-medium">{photo.fileName}</span>
        <span className="text-xs text-muted-foreground">
          {photo.capture.replace('T', ' ')}
          {photo.camera ? ` · ${photo.camera}` : ''}
        </span>
        <span className="ml-auto text-xs text-muted-foreground tabular-nums">
          {index + 1} / {photos.length}
        </span>
      </div>
      <div className="relative flex min-h-0 flex-1 items-center justify-center bg-muted/40 p-6">
        <img
          key={src}
          src={src}
          alt={photo.fileName}
          onLoad={() => setLoadedSrc(src)}
          className="max-h-full max-w-full rounded-sm object-contain shadow-lg"
        />
        {loadedSrc !== src && (
          <LoaderCircle className="absolute size-6 animate-spin text-muted-foreground" />
        )}
        <Button
          variant="secondary"
          size="icon"
          aria-label="Previous photo"
          className="absolute left-3"
          disabled={index === 0}
          onClick={() => onIndexChange(index - 1)}
        >
          <ChevronLeft />
        </Button>
        <Button
          variant="secondary"
          size="icon"
          aria-label="Next photo"
          className="absolute right-3"
          disabled={index >= photos.length - 1}
          onClick={() => onIndexChange(index + 1)}
        >
          <ChevronRight />
        </Button>
      </div>
    </div>
  )
}
