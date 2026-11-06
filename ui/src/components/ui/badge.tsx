import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "cn"
import { Slot } from "radix-ui"

const badgeVariants = cva(
  "group/badge inline-flex h-5 w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-4xl border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:size-3!",
  {
    variants: {
      variant: {
        // Solid and raised a step below a button (FR-099, STYLE_RULE §4 "Depth"): a 1.5 px lip in
        // a shade of the badge's own colour; it never reacts to the pointer.
        default: "bg-primary text-primary-foreground [--face:var(--primary)] chip-raised",
        secondary:
          "bg-secondary text-secondary-foreground [--face:var(--secondary)] chip-raised",
        destructive:
          "bg-danger-solid text-white [--face:var(--danger-solid)] chip-raised focus-visible:ring-destructive/20",
        // Deep colours with white text, the same in both themes (owner, 2026-11-06).
        success: "bg-success-solid text-white [--face:var(--success-solid)] chip-raised",
        warning: "bg-warning-solid text-white [--face:var(--warning-solid)] chip-raised",
        outline:
          "border-border bg-card text-foreground [--face:var(--card)] chip-raised",
        ghost:
          "hover:bg-muted hover:text-muted-foreground dark:hover:bg-muted/50",
        link: "text-primary underline-offset-4 hover:underline",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

function Badge({
  className,
  variant = "default",
  asChild = false,
  ...props
}: React.ComponentProps<"span"> &
  VariantProps<typeof badgeVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot.Root : "span"

  return (
    <Comp
      data-slot="badge"
      data-variant={variant}
      className={cn(badgeVariants({ variant }), className)}
      {...props}
    />
  )
}

export { Badge, badgeVariants }
