// SPDX-License-Identifier: GPL-3.0-only
import type { KeyboardEventHandler, PointerEventHandler, ReactNode, Ref } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { Separator } from '@/components/ui/separator'

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
  /** What the stage shows, on neutral black (STYLE_RULE §4). */
  stage: ReactNode
  /** A strip under the stage (the capture screen's shots). */
  below?: ReactNode
  title: ReactNode
  description: ReactNode
  descriptionHint?: string
  /** The inspector's sections. */
  children: ReactNode
  /** The keyboard hint line at the foot of the inspector. */
  footer: ReactNode
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
  stage,
  below,
  title,
  description,
  descriptionHint,
  children,
  footer,
}: ViewerFrameProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent
        ref={contentRef}
        onKeyDown={onKeyDown}
        onEscapeKeyDown={onEscapeKeyDown}
        className="flex h-[90vh] w-[95vw] max-w-none gap-0 overflow-hidden p-0 sm:max-w-[min(1400px,95vw)]"
      >
        <div className="flex min-w-0 flex-1 flex-col">
          {/* Stage */}
          <div
            ref={stageRef}
            className="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-neutral-950 p-8"
            onPointerDown={onStagePointerDown}
            onPointerMove={onStagePointerMove}
            onPointerUp={onStagePointerUp}
          >
            {stage}
          </div>
          {below}
        </div>

        {/* Inspector */}
        <aside className="flex w-96 shrink-0 flex-col border-l bg-background">
          <div className="space-y-0.5 px-5 pt-5 pb-4 pr-12">
            <DialogTitle className="truncate text-base">{title}</DialogTitle>
            <DialogDescription className="truncate text-xs" data-hint={descriptionHint}>
              {description}
            </DialogDescription>
          </div>
          <Separator />

          <div className="flex-1 space-y-5 overflow-y-auto p-5">{children}</div>

          <Separator />
          <p className="px-5 py-3 text-[11px] text-muted-foreground">{footer}</p>
        </aside>
      </DialogContent>
    </Dialog>
  )
}
