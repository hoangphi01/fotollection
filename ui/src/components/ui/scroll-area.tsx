"use client"

import * as React from "react"
import { cn } from "cn"
import { ScrollArea as ScrollAreaPrimitive } from "radix-ui"

function ScrollArea({
  className,
  children,
  gutter = false,
  ...props
}: React.ComponentProps<typeof ScrollAreaPrimitive.Root> & {
  /** Room on the right for the bar, 12 px, so a list inside a card keeps its words clear of
   * it (owner, 2026-11-08; UI-LAYOUT-018). A panel's own scroll has its cards' padding. */
  gutter?: boolean
}) {
  return (
    <ScrollAreaPrimitive.Root
      data-slot="scroll-area"
      // A column, so a `max-h-*` on the root reaches the viewport (`min-h-0 flex-1`): with
      // `size-full` alone its 100 % of an auto height grew with the rows and spilled over what
      // follows (LESSONS, UI-LAYOUT-017).
      className={cn("relative flex flex-col", gutter && "[&>[data-slot=scroll-area-viewport]>div]:pr-3", className)}
      {...props}
    >
      <ScrollAreaPrimitive.Viewport
        data-slot="scroll-area-viewport"
        className="min-h-0 w-full flex-1 rounded-[inherit] motion-control outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-1"
      >
        {children}
      </ScrollAreaPrimitive.Viewport>
      <ScrollBar />
      <ScrollAreaPrimitive.Corner />
    </ScrollAreaPrimitive.Root>
  )
}

function ScrollBar({
  className,
  orientation = "vertical",
  ...props
}: React.ComponentProps<typeof ScrollAreaPrimitive.ScrollAreaScrollbar>) {
  return (
    <ScrollAreaPrimitive.ScrollAreaScrollbar
      data-slot="scroll-area-scrollbar"
      data-orientation={orientation}
      orientation={orientation}
      className={cn(
        // One bar everywhere, as the gallery's (index.css `::-webkit-scrollbar`): 8 px with 2 px
        // clear on each side, a 4 px thumb (owner, 2026-11-08: "every scroll bar as small").
        "flex touch-none p-0.5 motion-control select-none data-horizontal:h-2 data-horizontal:flex-col data-vertical:h-full data-vertical:w-2",
        className
      )}
      {...props}
    >
      <ScrollAreaPrimitive.ScrollAreaThumb
        data-slot="scroll-area-thumb"
        className="relative flex-1 rounded-full bg-primary/70 hover:bg-primary"
      />
    </ScrollAreaPrimitive.ScrollAreaScrollbar>
  )
}

export { ScrollArea, ScrollBar }
