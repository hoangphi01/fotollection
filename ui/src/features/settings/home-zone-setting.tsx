// SPDX-License-Identifier: GPL-3.0-only
import { Globe } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { PickerTrigger } from '@/components/forms/picker-trigger'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { commands, type ZoneHit } from '@/ipc/bindings'
import { useSystemZone } from './use-system-zone'

const DEBOUNCE_MS = 120

/** `Asia/Ho_Chi_Minh · GMT+7`: the zone's name and its offset today. */
function zoneLabel(zone: string, language: string) {
  let offset = ''
  try {
    offset = new Intl.DateTimeFormat(language, { timeZone: zone, timeZoneName: 'shortOffset' }).formatToParts(new Date()).find((p) => p.type === 'timeZoneName')?.value ?? ''
  } catch {
    // A zone the web view does not know: its name alone.
  }
  return [zone.replaceAll('_', ' '), offset].filter(Boolean).join(' · ')
}

/** Zones by a country or a city typed (FR-145), searched in Rust over the bundled GeoNames. */
function useZoneSearch(query: string) {
  const [hits, setHits] = useState<ZoneHit[]>([])
  useEffect(() => {
    const text = query.trim()
    let current = true
    const timer = setTimeout(() => {
      if (text === '') return setHits([])
      void commands.zoneSearch(text).then((r) => {
        if (current && r.status === 'ok') setHits(r.data)
      })
    }, DEBOUNCE_MS)
    return () => {
      current = false
      clearTimeout(timer)
    }
  }, [query])
  return hits
}

/** A zone picked by typing a country or a city, as the place picker (FR-032). */
function ZonePicker({ label, onPick }: { label: string; onPick(zone: string): void }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const hits = useZoneSearch(query)
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <PickerTrigger icon={<Globe />} label={label} />
      </PopoverTrigger>
      <PopoverContent className="w-(--radix-popover-trigger-width) p-0" align="start">
        {/* The search is done in Rust, so cmdk's own filtering is off. */}
        <Command shouldFilter={false}>
          <CommandInput value={query} onValueChange={setQuery} placeholder={t('homeZone.search')} />
          <CommandList>
            {query.trim() !== '' && <CommandEmpty>{t('places.noMatch')}</CommandEmpty>}
            {hits.map((hit) => (
              <CommandItem
                key={hit.zone}
                value={hit.zone}
                onSelect={() => {
                  onPick(hit.zone)
                  setOpen(false)
                  setQuery('')
                }}
              >
                <span className="min-w-0 flex-1 truncate">
                  {hit.place}, {hit.country}
                </span>
                <span className="shrink-0 text-xs text-muted-foreground">{hit.zone.replaceAll('_', ' ')}</span>
              </CommandItem>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

/**
 * The home zone (FR-145): the computer's, or a place's chosen by its country or city. Clocks
 * without a zone are read in it; a short hint names the computer's zone when the two differ (a
 * laptop abroad). The same field on the welcome's first page and in Settings › General.
 */
export function HomeZoneSetting() {
  const { t, i18n } = useTranslation()
  const { ui, update } = useUiSettings()
  const system = useSystemZone()
  const home = ui.homeZone ?? system
  const [other, setOther] = useState(false)
  const away = ui.homeZone !== null && system !== null && ui.homeZone !== system
  const mode = other || away ? 'other' : 'computer'
  return (
    <Field
      label={t('homeZone.label')}
      hint={away && system ? t('homeZone.differs', { zone: system.replaceAll('_', ' ') }) : home ? t('homeZone.readIn', { zone: zoneLabel(home, i18n.language) }) : undefined}
    >
      <SegmentedToggle<'computer' | 'other'>
        value={mode}
        options={[
          { value: 'computer', label: t('homeZone.computer') },
          { value: 'other', label: t('homeZone.other') },
        ]}
        onChange={(v) => {
          setOther(v === 'other')
          if (v === 'computer' && system) void update({ homeZone: system })
        }}
        ariaLabel={t('homeZone.label')}
      />
      {mode === 'computer' ? null : <ZonePicker label={home ? zoneLabel(home, i18n.language) : t('homeZone.search')} onPick={(homeZone) => void update({ homeZone })} />}
    </Field>
  )
}
