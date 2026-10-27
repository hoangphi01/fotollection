// SPDX-License-Identifier: GPL-3.0-only
import { MapPin } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import type { Place } from '@/ipc/bindings'
import { usePlaceSearch } from './use-place-search'

type PlacePickerProps = {
  /** The text on the button: the chosen place, or a prompt. */
  label: string | null
  onPick: (place: Place) => void
}

/**
 * Pick a town or city by typing its name (FR-032): diacritics and spaces are ignored, so
 * "ha noi" finds Hà Nội. The same picker everywhere a place is set.
 */
export function PlacePicker({ label, onPick }: PlacePickerProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const places = usePlaceSearch(query)
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" role="combobox" aria-expanded={open} className="w-full justify-start font-normal">
          <MapPin data-icon="inline-start" />
          <span className="truncate">{label ?? t('places.pick')}</span>
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-(--radix-popover-trigger-width) p-0" align="start">
        {/* The search is done in Rust, so cmdk's own filtering is off. */}
        <Command shouldFilter={false}>
          <CommandInput value={query} onValueChange={setQuery} placeholder={t('places.searchPlaceholder')} />
          <CommandList>
            {query.trim() !== '' && <CommandEmpty>{t('places.noMatch')}</CommandEmpty>}
            {places.map((place) => (
              <CommandItem
                key={place.geonamesId}
                value={String(place.geonamesId)}
                onSelect={() => {
                  onPick(place)
                  setOpen(false)
                  setQuery('')
                }}
              >
                <span className="min-w-0 flex-1 truncate">{place.name}</span>
                <span className="shrink-0 text-xs text-muted-foreground">{place.country}</span>
              </CommandItem>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
