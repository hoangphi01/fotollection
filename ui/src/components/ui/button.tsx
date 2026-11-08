import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "cn"
import { Slot } from "radix-ui"

const buttonVariants = cva(
  "group/button inline-flex shrink-0 items-center justify-center rounded-lg border border-transparent bg-clip-padding text-sm font-medium whitespace-nowrap motion-control outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 active:not-aria-[haspopup]:translate-y-px disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      variant: {
        // Raised (FR-099, STYLE_RULE §4 "Depth"): a lip in a shade of the button's own colour.
        // While held, every button but a red one or a link is the theme's colour pressed in, and
        // rises again on release, so a press is always seen (owner, 2026-11-08). The press keeps the
        // fast pace on `ease-enter` (most of the colour in the first 40 ms, so a quick click shows it;
        // a cut with no motion felt jerky) and the release `ease-move`, as every control. Marked important: in dark,
        // `dark:bg-secondary` and `dark:hover:bg-muted` weigh as much or more and come later. Ghost and link
        // stay flat at rest.
        default: "bg-primary text-primary-foreground [--face:var(--primary)] raised hover:bg-primary-deep active:bg-pressed! active:text-primary-foreground! active:[--face:var(--primary)]! active:pressed-in! active:ease-enter aria-pressed:bg-pressed aria-pressed:pressed-in aria-pressed:hover:bg-primary",
        outline:
          "border-border bg-card [--face:var(--card)] raised hover:bg-muted hover:text-foreground active:bg-pressed! active:text-primary-foreground! active:[--face:var(--primary)]! active:pressed-in! active:ease-enter aria-expanded:bg-muted aria-expanded:text-foreground dark:border-input dark:bg-secondary dark:[--face:var(--secondary)] dark:hover:bg-muted",
        secondary:
          "bg-secondary text-secondary-foreground [--face:var(--secondary)] raised hover:bg-[color-mix(in_oklch,var(--secondary),var(--foreground)_5%)] active:bg-pressed! active:text-primary-foreground! active:[--face:var(--primary)]! active:pressed-in! active:ease-enter aria-expanded:bg-secondary aria-expanded:text-secondary-foreground",
        ghost:
          "hover:bg-muted hover:text-foreground active:bg-pressed! active:text-primary-foreground! active:[--face:var(--primary)]! active:pressed-in! active:ease-enter aria-expanded:bg-muted aria-expanded:text-foreground dark:hover:bg-muted/50",
        // Red for anything that deletes, clears, removes or stops (owner, 2026-11-06; STYLE_RULE
        // §4 "Red"): the red badges' deep red in both themes, white ink, their burgundy lip,
        // deeper under the pointer; icon-only ones too (a tab's ✕ is a round one).
        destructive:
          "bg-danger-solid text-white [--face:var(--danger-solid)] [--lip-color:var(--danger-lip)] raised-lip active:not-aria-[haspopup]:raised-lip-pressed hover:bg-[color-mix(in_oklch,var(--danger-solid),black_25%)] focus-visible:border-destructive/40 focus-visible:ring-destructive/30",
        // An action on a coloured surface (a green or red notice): the theme's accent in dark,
        // where the wood stands out on green; the card's cream in light, where the teal accent
        // would sink into the green (owner, 2026-11-06). Raised as every button.
        "on-colour":
          "bg-card text-foreground [--face:var(--card)] raised hover:bg-muted active:bg-pressed! active:text-primary-foreground! active:[--face:var(--primary)]! active:pressed-in! active:ease-enter dark:bg-primary dark:text-primary-foreground dark:[--face:var(--primary)] dark:hover:bg-primary/90",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default:
          "h-8 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
        xs: "h-6 gap-1 rounded-[min(var(--radius-md),10px)] px-2 text-xs in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3",
        sm: "h-8 gap-1 rounded-[min(var(--radius-md),12px)] px-2.5 text-xs in-data-[slot=button-group]:rounded-lg has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-4",
        lg: "h-8 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
        icon: "size-8",
        "icon-xs":
          "size-6 rounded-[min(var(--radius-md),10px)] in-data-[slot=button-group]:rounded-lg [&_svg:not([class*='size-'])]:size-3",
        "icon-sm":
          "size-8 rounded-[min(var(--radius-md),12px)] in-data-[slot=button-group]:rounded-lg",
        "icon-lg": "size-8",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

function Button({
  className,
  variant = "default",
  size = "default",
  asChild = false,
  ...props
}: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean
  }) {
  const Comp = asChild ? Slot.Root : "button"

  return (
    <Comp
      data-slot="button"
      data-variant={variant}
      data-size={size}
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  )
}

export { Button, buttonVariants }
