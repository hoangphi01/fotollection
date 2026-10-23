// SPDX-License-Identifier: GPL-3.0-only
import { CalendarDays, Crop, Download, MapPin } from 'lucide-react'
import { useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { commands } from '@/ipc/bindings'
import { useIpcQuery } from './use-ipc-query'

const DEFAULT_PRINT_SIZE = '10x15'

function formatCm(mm: number): string {
  return String(mm / 10)
}

export function SettingsPanel() {
  const printSizes = useIpcQuery(commands.listPrintSizes)
  const appInfo = useIpcQuery(commands.appInfo)
  const [printSize, setPrintSize] = useState(DEFAULT_PRINT_SIZE)
  const [dateStamp, setDateStamp] = useState(true)
  const [locationStamp, setLocationStamp] = useState(true)

  return (
    <aside className="flex w-72 shrink-0 flex-col border-l">
      <div className="flex h-12 items-center px-4">
        <span className="text-sm font-semibold">Print settings</span>
      </div>
      <Separator />

      <div className="flex-1 space-y-6 overflow-y-auto p-4">
        <section className="space-y-2">
          <Label htmlFor="print-size" className="flex items-center gap-2">
            <Crop className="size-4 text-muted-foreground" />
            Print size
          </Label>
          <Select value={printSize} onValueChange={setPrintSize}>
            <SelectTrigger id="print-size" className="w-full">
              <SelectValue placeholder="Loading…" />
            </SelectTrigger>
            <SelectContent>
              {printSizes?.map((size) => (
                <SelectItem key={size.id} value={size.id}>
                  {formatCm(size.shortMm)} × {formatCm(size.longMm)} cm
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </section>

        <section className="space-y-3">
          <span className="text-xs font-medium text-muted-foreground uppercase">Stamps</span>
          <div className="flex items-center justify-between">
            <Label htmlFor="date-stamp" className="flex items-center gap-2">
              <CalendarDays className="size-4 text-muted-foreground" />
              Date
            </Label>
            <Switch id="date-stamp" checked={dateStamp} onCheckedChange={setDateStamp} />
          </div>
          <div className="flex items-center justify-between">
            <Label htmlFor="location-stamp" className="flex items-center gap-2">
              <MapPin className="size-4 text-muted-foreground" />
              Location
            </Label>
            <Switch
              id="location-stamp"
              checked={locationStamp}
              onCheckedChange={setLocationStamp}
            />
          </div>
        </section>
      </div>

      <Separator />
      <div className="space-y-3 p-4">
        <Tooltip>
          <TooltipTrigger asChild>
            {/* span wrapper: disabled buttons don't emit pointer events for the tooltip */}
            <span className="block">
              <Button className="w-full" disabled>
                <Download data-icon="inline-start" />
                Export
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent>Export arrives in Phase 1</TooltipContent>
        </Tooltip>
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>Offline · no telemetry</span>
          {appInfo && <Badge variant="outline">v{appInfo.appVersion}</Badge>}
        </div>
      </div>
    </aside>
  )
}
