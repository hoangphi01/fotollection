// SPDX-License-Identifier: GPL-3.0-only
import type { KeyboardEventHandler, MouseEvent, PointerEventHandler, ReactNode, Ref } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'

type DivRef = Ref<HTMLDivElement>
type StagePointer = PointerEventHandler<HTMLDivElement>
type FrameKeys = KeyboardEventHandler<HTMLDivElement>

type ViewerFrameProps = {
  open: boolean
  onClose: () => void
  contentRef?: DivRef
  onKeyDown?: FrameKeys
  /** Called before Radix closes the dialog on Esc; `preventDefault` keeps it open. */
  onEscapeKeyDown?: (event: globalThis.KeyboardEvent) => void
  stageRef?: DivRef
  onStagePointerDown?: StagePointer
  onStagePointerMove?: StagePointer
  onStagePointerUp?: StagePointer
  /** A click on the stage's own ground, around what it shows. */
  onStageGroundClick?: () => void
  /** What the stage shows, on neutral black (STYLE_RULE §4). */
  stage: ReactNode
  /** A row of tools above the stage, so nothing sits over the photo (the photo preview, FR-095). */
  toolbar?: ReactNode
  /** A strip under the stage (the capture screen's shots). */
  below?: ReactNode
  title: ReactNode
  description: ReactNode
  descriptionHint?: string
  /** The inspector's sections. */
  children: ReactNode
  /** The keyboard hint line at the foot of the inspector. */
  footer: ReactNode
  /** `window` fills the app window, below the Windows title bar, instead of a dialog (the
   * photo preview, FR-095). */
  layout?: 'dialog' | 'window'
  /** A column left of the stage (the photo preview's Navigator and History, FR-095). */
  side?: ReactNode
  /** False hides the side column and the inspector, as Lightroom's Tab does. */
  panels?: boolean
  /** The inspector's tabs, under its title (the photo preview's Edit, Crop, Print, Info). */
  tabs?: ReactNode
  /** A card heading the inspector on every tab, above the scrolling sections (the histogram
   * in the dialog layout, which has no side column, FR-098). */
  inspectorTop?: ReactNode
  /** The side column over the stage rather than beside it: a narrow window, where three
   * columns would leave the photo too small (FR-095). */
  sideOver?: boolean
}

/**
 * The frame shared by the photo preview and the capture screen: the same dialog size, a
 * neutral stage, and an inspector with a title, scrolling sections and a hint line, so the
 * two cannot drift apart (STYLE_RULE §4, ADR-0028).
 */
export function ViewerFrame({
  open,
  onClose,
  contentRef,
  onKeyDown,
  onEscapeKeyDown,
  stageRef,
  onStagePointerDown,
  onStagePointerMove,
  onStagePointerUp,
  onStageGroundClick,
  stage,
  toolbar,
  below,
  title,
  description,
  descriptionHint,
  children,
  footer,
  layout = 'dialog',
  side,
  panels = true,
  tabs,
  inspectorTop,
  sideOver = false,
}: ViewerFrameProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent
        ref={contentRef}
        onKeyDown={onKeyDown}
        onEscapeKeyDown={onEscapeKeyDown}
        className={cn(
          'flex max-w-none gap-0 overflow-hidden p-0',
          layout === 'window'
            ? // The whole window under the Windows title bar (`--titlebar`, FR-052), square.
              'top-(--titlebar,0px) left-0 h-[calc(100vh-var(--titlebar,0px))] w-screen translate-x-0 translate-y-0 rounded-none ring-0 sm:max-w-none data-open:zoom-in-100 data-closed:zoom-out-100'
            : 'h-[90vh] w-[95vw] sm:max-w-[min(1400px,95vw)]',
        )}
      >
        {side && panels && (
          // Cards spaced as the inspector's, 12 px (STYLE_RULE §4).
          <aside
            className={cn(
              'flex w-panel shrink-0 flex-col border-r bg-background',
              sideOver && 'absolute inset-y-0 left-0 z-20 lifted-strong',
            )}
          >
            <ScrollArea className="min-h-0 flex-1 [&>[data-slot=scroll-area-viewport]>div]:block!">
              <div className="space-y-3 p-3">{side}</div>
            </ScrollArea>
          </aside>
        )}
        <div className="flex min-w-0 flex-1 flex-col">
          {/* The row owns no border; with the inspector hidden, the close button sits at its end. */}
          {toolbar && <div className={cn('shrink-0 border-b bg-background', !panels && 'pr-10')}>{toolbar}</div>}
          {/* Stage */}
          <div
            ref={stageRef}
            className="group/stage relative flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-stage p-8"
            onPointerDown={onStagePointerDown}
            onPointerMove={onStagePointerMove}
            onPointerUp={onStagePointerUp}
            onClick={(e: MouseEvent<HTMLDivElement>) => e.target === e.currentTarget && onStageGroundClick?.()}
          >
            {stage}
          </div>
          {below}
        </div>

        {/* Inspector */}
        <aside className={cn('flex w-panel shrink-0 flex-col border-l bg-background', !panels && 'hidden')}>
          {/* As tall as the toolbar row beside it, its line meeting the toolbar's (owner,
              2026-11-06: one line across the columns). */}
          <div className="box-content flex h-header shrink-0 flex-col justify-center gap-0.5 border-b px-4 pr-12">
            <DialogTitle className="truncate text-base">{title}</DialogTitle>
            <DialogDescription className="truncate text-xs" data-hint={descriptionHint}>
              {description}
            </DialogDescription>
          </div>
          {tabs && (
            <>
              {tabs}
              <Separator />
            </>
          )}

          {inspectorTop && (
            <>
              <div className="p-3">{inspectorTop}</div>
              <Separator />
            </>
          )}
          {/* Cards spaced as in the side panel, 12 px (STYLE_RULE §4, the lego rule). */}
          <ScrollArea className="min-h-0 flex-1 [&>[data-slot=scroll-area-viewport]>div]:block!">
            <div className="space-y-3 p-3">{children}</div>
          </ScrollArea>

          <Separator />
          <p className="px-4 py-3 text-xs text-muted-foreground">{footer}</p>
        </aside>
      </DialogContent>
    </Dialog>
  )
}
