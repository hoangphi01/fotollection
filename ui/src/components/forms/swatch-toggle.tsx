// SPDX-License-Identifier: GPL-3.0-only
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { cn } from '@/lib/utils'

export type SwatchOption<T extends string> = {
  value: T
  label: string
  /** Tailwind classes for the dot, or a CSS colour for one chosen by the user. */
  swatch: string
}

type SwatchToggleProps<T extends string> = {
  value: T | undefined
  options: readonly SwatchOption<T>[]
  onChange: (value: T) => void
  ariaLabel?: string
  disabled?: boolean
  /** Tighter items, for four colours in one row. */
  dense?: boolean
  className?: string
}

/**
 * A row of colours, each with its dot, 32 px tall like every control in a panel: the border's, the stamp's, the frame's, the postcard
 * ink's and a shared photo's fill (FR-067, FR-004, FR-040, FR-073, FR-075). One control, so
 * every colour choice looks and behaves the same.
 */
export function SwatchToggle<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  disabled = false,
  dense = false,
  className = 'w-full',
}: SwatchToggleProps<T>) {
  return (
    <ToggleGroup
      type="single"
      variant="outline"
      className={className}
      aria-label={ariaLabel}
      disabled={disabled}
      value={value ?? ''}
      onValueChange={(v) => v && onChange(v as T)}
    >
      {options.map((o) => {
        const css = o.swatch.startsWith('#') || o.swatch.startsWith('rgb')
        return (
          <ToggleGroupItem key={o.value} value={o.value} className={cn('min-w-0 flex-1 gap-1.5 px-1.5 text-xs', dense && 'px-1')}>
            <span
              className={cn('size-3 shrink-0 rounded-full border', !css && o.swatch)}
              style={css ? { backgroundColor: o.swatch } : undefined}
            />
            {o.label}
          </ToggleGroupItem>
        )
      })}
    </ToggleGroup>
  )
}
