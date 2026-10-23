// SPDX-License-Identifier: GPL-3.0-only
import { PanelBottom } from 'lucide-react'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { FrameColor, FrameDateFormat, FrameFields, FrameFit } from '@/ipc/bindings'
import { FontSelect } from '@/features/fonts/font-select'
import type { FontsApi } from '@/features/fonts/use-fonts'
import type { FrameConfigApi } from './use-frame-config'

const FIELDS: { key: keyof FrameFields; label: string; disabledHint?: string }[] = [
  { key: 'logo', label: 'Logo' },
  { key: 'camera', label: 'Camera' },
  { key: 'lens', label: 'Lens' },
  { key: 'focal', label: 'Focal length' },
  { key: 'aperture', label: 'Aperture' },
  { key: 'shutter', label: 'Shutter' },
  { key: 'iso', label: 'ISO' },
  { key: 'photoStyle', label: 'Photo style' },
  { key: 'date', label: 'Date' },
  { key: 'place', label: 'Place', disabledHint: 'Needs offline place names (FR-005)' },
]

type FrameSectionProps = {
  frame: FrameConfigApi
  fonts: FontsApi
}

/** "Shot on …" info frame settings, LUMIX Lab style (FR-040). */
export function FrameSection({ frame, fonts }: FrameSectionProps) {
  const settings = frame.config?.settings
  const enabled = settings?.enabled ?? false

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <Label htmlFor="info-frame" className="flex items-center gap-2">
          <PanelBottom className="size-4 text-muted-foreground" />
          Info frame
        </Label>
        <Switch
          id="info-frame"
          checked={enabled}
          onCheckedChange={(on) => void frame.setSettings((s) => ({ ...s, enabled: on }))}
        />
      </div>

      <div className={enabled ? 'space-y-3' : 'pointer-events-none space-y-3 opacity-50'}>
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={settings?.fit ?? ''}
          onValueChange={(fit) => fit && void frame.setSettings((s) => ({ ...s, fit: fit as FrameFit }))}
        >
          <ToggleGroupItem value="crop" className="flex-1 text-xs">
            Crop to fill
          </ToggleGroupItem>
          <ToggleGroupItem value="fit" className="flex-1 text-xs">
            Fit whole photo
          </ToggleGroupItem>
        </ToggleGroup>

        <div className="flex gap-2">
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            className="flex-1"
            value={settings?.color ?? ''}
            onValueChange={(color) =>
              color && void frame.setSettings((s) => ({ ...s, color: color as FrameColor }))
            }
          >
            <ToggleGroupItem value="white" className="flex-1 gap-1.5 text-xs">
              <span className="size-3 rounded-full border bg-white" /> White
            </ToggleGroupItem>
            <ToggleGroupItem value="black" className="flex-1 gap-1.5 text-xs">
              <span className="size-3 rounded-full border bg-neutral-900" /> Black
            </ToggleGroupItem>
          </ToggleGroup>
        </div>

        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="w-full"
          value={settings?.dateFormat ?? ''}
          onValueChange={(f) =>
            f && void frame.setSettings((s) => ({ ...s, dateFormat: f as FrameDateFormat }))
          }
        >
          <ToggleGroupItem value="slash" className="flex-1 font-mono text-[11px]">
            2026/10/18
          </ToggleGroupItem>
          <ToggleGroupItem value="iso" className="flex-1 font-mono text-[11px]">
            2026-10-18
          </ToggleGroupItem>
          <ToggleGroupItem value="dmy" className="flex-1 font-mono text-[11px]">
            18.10.2026
          </ToggleGroupItem>
        </ToggleGroup>

        <FontSelect
          fonts={fonts}
          label="Frame font"
          value={settings?.font}
          onChange={(font) => void frame.setSettings((s) => ({ ...s, font }))}
        />

        <div className="space-y-1.5">
          <span className="text-xs font-medium text-muted-foreground uppercase">Show</span>
          <div className="grid grid-cols-2 gap-x-3 gap-y-2">
            {FIELDS.map(({ key, label, disabledHint }) => (
              <label
                key={key}
                title={disabledHint}
                className="flex items-center gap-2 text-sm"
                aria-disabled={Boolean(disabledHint)}
              >
                <Checkbox
                  checked={settings?.fields[key] ?? false}
                  disabled={Boolean(disabledHint)}
                  onCheckedChange={(checked) =>
                    void frame.setSettings((s) => ({
                      ...s,
                      fields: { ...s.fields, [key]: checked === true },
                    }))
                  }
                />
                <span className={disabledHint ? 'text-muted-foreground' : ''}>{label}</span>
              </label>
            ))}
          </div>
        </div>
        <p className="text-xs text-muted-foreground">
          The whole print, frame included, keeps the selected size. Edit camera, lens and style in
          the preview.
        </p>
      </div>
    </section>
  )
}
