import * as React from "react"
import { cn } from "cn"
import { Dialog as DialogPrimitive } from "radix-ui"

import { Button } from "@/components/ui/button"
import { XIcon } from "lucide-react"

function Dialog({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Root>) {
  return <DialogPrimitive.Root data-slot="dialog" {...props} />
}

function DialogTrigger({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Trigger>) {
  return <DialogPrimitive.Trigger data-slot="dialog-trigger" {...props} />
}

function DialogPortal({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Portal>) {
  return <DialogPrimitive.Portal data-slot="dialog-portal" {...props} />
}

function DialogClose({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Close>) {
  return <DialogPrimitive.Close data-slot="dialog-close" {...props} />
}

function DialogOverlay({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Overlay>) {
  return (
    <DialogPrimitive.Overlay
      data-slot="dialog-overlay"
      className={cn(
        "fixed inset-0 isolate z-50 bg-black/10 supports-backdrop-filter:backdrop-blur-xs data-open:animate-in data-open:motion-in-slow data-open:fade-in-0 data-closed:animate-out data-closed:motion-out-slow data-closed:fade-out-0",
        className
      )}
      {...props}
    />
  )
}

/** A press on the window's own bar (`data-window-chrome`: moving, resizing or closing the
 * window), which an open dialog must let through and never take for a click outside it
 * (owner, 2026-11-08: "the window cannot be moved while the preview is open"). */
function onWindowChrome(event: { target: EventTarget | null; detail?: { originalEvent?: Event } }): boolean {
  const target = event.detail?.originalEvent?.target ?? event.target
  return target instanceof Element && target.closest("[data-window-chrome]") !== null
}

function DialogContent({
  className,
  children,
  showCloseButton = true,
  onPointerDownOutside,
  onInteractOutside,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Content> & {
  showCloseButton?: boolean
}) {
  return (
    <DialogPortal>
      <DialogOverlay />
      <DialogPrimitive.Content
        data-slot="dialog-content"
        className={cn(
          "fixed top-1/2 left-1/2 z-50 grid w-full max-w-[calc(100%-2rem)] -translate-x-1/2 -translate-y-1/2 gap-4 rounded-xl bg-popover p-4 lifted-strong text-sm text-popover-foreground ring-1 ring-foreground/10 outline-none sm:max-w-lg data-open:animate-in data-open:motion-in-slow data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:motion-out-slow data-closed:fade-out-0 data-closed:zoom-out-95",
          className
        )}
        onPointerDownOutside={(e) => {
          if (onWindowChrome(e)) e.preventDefault()
          onPointerDownOutside?.(e)
        }}
        onInteractOutside={(e) => {
          if (onWindowChrome(e)) e.preventDefault()
          onInteractOutside?.(e)
        }}
        {...props}
      >
        {children}
        {showCloseButton && (
          <DialogPrimitive.Close data-slot="dialog-close" asChild>
            {/* A small red square with even corners, so closing is easy to find (FR-052); it
                deepens to burgundy under the pointer, in both themes. */}
            <Button
              variant="destructive"
              className="absolute top-4 right-4 size-8 rounded-md [--lip:3px]"
              size="icon-sm"
            >
              <XIcon
              />
              <span className="sr-only">Close</span>
            </Button>
          </DialogPrimitive.Close>
        )}
      </DialogPrimitive.Content>
    </DialogPortal>
  )
}

function DialogHeader({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="dialog-header"
      // Clear of the close button (32 px at 16 px from the edge) by 8 px, so a long title or
      // description never runs under it (owner, 2026-11-08; UI-LAYOUT-022).
      className={cn("flex flex-col gap-2 pr-10", className)}
      {...props}
    />
  )
}

function DialogFooter({
  className,
  showCloseButton = false,
  children,
  ...props
}: React.ComponentProps<"div"> & {
  showCloseButton?: boolean
}) {
  return (
    <div
      data-slot="dialog-footer"
      className={cn(
        "-mx-4 -mb-4 flex flex-col-reverse gap-2 rounded-b-xl border-t bg-muted/50 p-4 sm:flex-row sm:flex-wrap sm:justify-end",
        className
      )}
      {...props}
    >
      {children}
      {showCloseButton && (
        <DialogPrimitive.Close asChild>
          <Button variant="outline">Close</Button>
        </DialogPrimitive.Close>
      )}
    </div>
  )
}

/**
 * A dialog's form: its fields and its footer spaced as the dialog's own blocks, 16 px, so a
 * field never touches the footer's line (owner, 2026-11-08: "the dialog is stuck"). A `form`
 * inside a dialog is this one (`check-lego`).
 */
function DialogForm({ className, ...props }: React.ComponentProps<"form">) {
  return <form data-slot="dialog-form" className={cn("grid gap-4", className)} {...props} />
}

function DialogTitle({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Title>) {
  return (
    <DialogPrimitive.Title
      data-slot="dialog-title"
      className={cn(
        "font-heading text-base leading-snug font-medium",
        className
      )}
      {...props}
    />
  )
}

function DialogDescription({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Description>) {
  return (
    <DialogPrimitive.Description
      data-slot="dialog-description"
      className={cn(
        "text-sm text-muted-foreground *:[a]:underline *:[a]:underline-offset-3 *:[a]:hover:text-foreground",
        className
      )}
      {...props}
    />
  )
}

export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogForm,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
}
