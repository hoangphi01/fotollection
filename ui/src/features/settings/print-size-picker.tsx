// SPDX-License-Identifier: GPL-3.0-only
import { Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { CustomPrintSize, LengthUnit, PrintSize, RenderSettings } from '@/ipc/bindings'
import { ratioLabel } from '@/lib/crop-geometry'
import { MM_PER_IN, sizeLabel } from '@/lib/print-size'

const ADD = '__add__'
/** Keeps each photo's own ratio, no crop (FR-045). Matches `PrintSize::ORIGINAL_ID`. */
const ORIGINAL = 'original'

type PrintSizePickerProps = {
  settings: RenderSettings | null
  builtIn: PrintSize[] | undefined
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/** Built-in and saved custom print sizes, with an inline "add custom size" form (FR-043). */
export function PrintSizePicker({ settings, builtIn, onChange }: PrintSizePickerProps) {
  const custom = settings?.customSizes ?? []
  const [adding, setAdding] = useState(false)
  const [name, setName] = useState('')
  const [width, setWidth] = useState('')
  const [height, setHeight] = useState('')
  const [unit, setUnit] = useState<LengthUnit>('in')
  const [formError, setFormError] = useState<string | null>(null)
  const selectedCustom = custom.find((c) => c.id === settings?.printSize)

  const save = () => {
    const parse = (s: string) => Number.parseFloat(s.replace(',', '.'))
    const [a, b] = [parse(width), parse(height)]
    const toMm = (v: number) => Math.round(unit === 'in' ? v * MM_PER_IN : v * 10)
    if (!(a > 0) || !(b > 0)) return setFormError('Enter both edges')
    const [shortMm, longMm] = [toMm(Math.min(a, b)), toMm(Math.max(a, b))]
    if (shortMm < 20 || longMm > 1000) return setFormError('Edges must be between 2 and 100 cm')
    const next = Math.max(0, ...custom.map((c) => Number(c.id.replace('custom-', '')) || 0)) + 1
    const size: CustomPrintSize = {
      id: `custom-${next}`,
      name: name.trim() || sizeLabel(shortMm, longMm, unit),
      shortMm,
      longMm,
      unit,
    }
    onChange((s) => ({ ...s, customSizes: [...(s.customSizes ?? []), size], printSize: size.id }))
    setAdding(false)
    setName('')
    setWidth('')
    setHeight('')
    setFormError(null)
  }

  const remove = (id: string) =>
    onChange((s) => ({
      ...s,
      customSizes: (s.customSizes ?? []).filter((c) => c.id !== id),
      printSize: s.printSize === id ? '10x15' : s.printSize,
    }))

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <Select
          value={settings?.printSize ?? ''}
          onValueChange={(v) => {
            if (v === ADD) setAdding(true)
            else onChange((s) => ({ ...s, printSize: v }))
          }}
        >
          <SelectTrigger id="print-size" className="w-full">
            <SelectValue placeholder="Loading…" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value={ORIGINAL}>
                Original
                <span className="text-muted-foreground"> · photo's own ratio, no crop</span>
              </SelectItem>
            </SelectGroup>
            <SelectSeparator />
            <SelectGroup>
              <SelectLabel>Standard (CEWE / EU)</SelectLabel>
              {builtIn?.map((size) => (
                <SelectItem key={size.id} value={size.id}>
                  {sizeLabel(size.shortMm, size.longMm)}
                  <span className="text-muted-foreground">
                    {' '}
                    · {ratioLabel(size.shortMm, size.longMm)}
                  </span>
                </SelectItem>
              ))}
            </SelectGroup>
            {custom.length > 0 && (
              <SelectGroup>
                <SelectLabel>Custom</SelectLabel>
                {custom.map((c) => (
                  <SelectItem key={c.id} value={c.id}>
                    {c.name}
                    <span className="text-muted-foreground">
                      {' '}
                      · {sizeLabel(c.shortMm, c.longMm, c.unit ?? 'cm')} ·{' '}
                      {ratioLabel(c.shortMm, c.longMm)}
                    </span>
                  </SelectItem>
                ))}
              </SelectGroup>
            )}
            <SelectSeparator />
            <SelectItem value={ADD}>
              <Plus className="size-3.5" /> Custom size…
            </SelectItem>
          </SelectContent>
        </Select>
        {selectedCustom && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={`Delete ${selectedCustom.name}`}
            title="Delete this custom size"
            onClick={() => remove(selectedCustom.id)}
          >
            <Trash2 />
          </Button>
        )}
      </div>

      {adding && (
        <div className="space-y-2 rounded-lg border bg-muted/30 p-3">
          <Input
            placeholder="Name (e.g. Instax wide)"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="h-8 text-sm"
          />
          <div className="flex items-center gap-2">
            <Input
              placeholder="4"
              inputMode="decimal"
              value={width}
              onChange={(e) => setWidth(e.target.value)}
              className="h-8 text-sm"
              aria-label="Width"
            />
            <span className="text-muted-foreground">×</span>
            <Input
              placeholder="6"
              inputMode="decimal"
              value={height}
              onChange={(e) => setHeight(e.target.value)}
              className="h-8 text-sm"
              aria-label="Height"
            />
            <ToggleGroup
              type="single"
              variant="outline"
              size="sm"
              value={unit}
              onValueChange={(u) => u && setUnit(u as LengthUnit)}
            >
              <ToggleGroupItem value="cm" className="text-xs">
                cm
              </ToggleGroupItem>
              <ToggleGroupItem value="in" className="text-xs">
                in
              </ToggleGroupItem>
            </ToggleGroup>
          </div>
          {formError && <p className="text-xs text-destructive">{formError}</p>}
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setAdding(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={save}>
              Save size
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
