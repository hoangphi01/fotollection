// SPDX-License-Identifier: GPL-3.0-only
import { MapPin } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { commands, type CountryMap, type CountryFacet, type SortPick } from '@/ipc/bindings'

type CountryMapViewProps = {
  country: CountryFacet
  /** The tab chosen (FR-141): its photos' places only. */
  folders: number[]
  picks: SortPick[]
  onPicks(picks: SortPick[]): void
}

/**
 * One country of the roll, its outline and a pin per place with photos (FR-141; owner
 * 2026-11-18: "a filter app, not Google Maps"): a pin ticks its places as their rows do, and
 * close towns share one. Drawn by the core (`geo_shapes`), offline; no map library.
 */
export function CountryMapView({ country, folders, picks, onPicks }: CountryMapViewProps) {
  const { t, i18n } = useTranslation()
  const [map, setMap] = useState<CountryMap | null>(null)
  const scope = JSON.stringify(folders)
  useEffect(() => {
    void commands.countryMap(JSON.parse(scope) as number[], country.country).then((r) => setMap(r.status === 'ok' ? r.data : null))
  }, [scope, country.country, country.count])
  if (!map) return null
  // Floats cross IPC as `number | null` (a NaN would be null); the core never sends one.
  const width = map.width ?? 1
  const height = map.height ?? 1
  const name = (id: number) => country.places.find((p) => p.id === id)?.name ?? ''
  const picked = (id: number) => picks.some((p) => p.kind === 'place' && p.id === id)
  return (
    <div className="relative w-full" style={{ aspectRatio: `${width} / ${height}` }}>
      <svg viewBox={`0 0 ${width} ${height}`} className="absolute inset-0 size-full" role="img" aria-label={country.country}>
        <path d={map.path} className="fill-(--map-land) stroke-(--map-land)" strokeWidth={0.75} strokeLinejoin="round" />
      </svg>
      {map.pins.map((pin) => {
        const on = pin.ids.every(picked)
        const label = `${pin.ids.map(name).join(', ')} · ${pin.count.toLocaleString(i18n.language)}`
        return (
          <span
            key={pin.ids.join('-')}
            data-hint={label}
            className="absolute -translate-x-1/2 -translate-y-1/2"
            style={{ left: `${((pin.x ?? 0) / width) * 100}%`, top: `${((pin.y ?? 0) / height) * 100}%` }}
          >
            {/* A 32 px target holding RollSense's violet key with a pin, pressed in once ticked. */}
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('rollsense.pin', { places: label })}
              aria-pressed={on}
              onClick={() =>
                onPicks(
                  on
                    ? picks.filter((p) => !(p.kind === 'place' && pin.ids.includes(p.id)))
                    : [...picks, ...pin.ids.filter((id) => !picked(id)).map((id) => ({ kind: 'place' as const, id }))],
                )
              }
            >
              <span aria-hidden data-slot="sense-key">
                <MapPin />
              </span>
            </Button>
          </span>
        )
      })}
    </div>
  )
}
