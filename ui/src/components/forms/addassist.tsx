// SPDX-License-Identifier: GPL-3.0-only
import { Languages, MapPin } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands, type Address, type Country, type CountryInfo, type Found } from '@/ipc/bindings'
import { emptyAddress } from '@/lib/address'
import { CommitInput } from './commit-input'
import { Field, Hint, RowList, Stack } from './field'
import { ListRow } from './list-row'
import { SegmentedToggle } from './segmented-toggle'
import { SwitchField } from './switch-field'

/** How many places a search shows. */
const RESULTS = 8

let countries: Promise<CountryInfo[]> | null = null
/** AddAssist's countries, asked once. */
function countryInfo(country: Country): Promise<CountryInfo | undefined> {
  countries ??= commands.addassistCountries().then((r) => (r.status === 'ok' ? r.data : []))
  return countries.then((all) => all.find((c) => c.country === country))
}

type AddAssistProps = {
  country: Country
  value: Address | null
  /** Called with the whole address when a field is left or a place picked (a draft never). */
  onCommit: (address: Address) => void
  /** By hand or assisted, one switch for every address field of the machine. */
  assisted: boolean
  onAssisted: (assisted: boolean) => void
}

/**
 * AddAssist (FR-133), the address field of every Japanese or Korean card: by hand, every line
 * typed; assisted, a postal code or a search fills what the post's data knows (the prefecture,
 * municipality and town; the 시·도, 시·군·구 and road), looked up offline in Rust, every filled
 * field still editable. The building number, the name and the honorific are typed. The one
 * block for an address: `check-lego` refuses AddAssist's commands anywhere else.
 */
export function AddAssist({ country, value, onCommit, assisted, onAssisted }: AddAssistProps) {
  const { t } = useTranslation()
  const address = value ?? emptyAddress(country)
  const [found, setFound] = useState<Found[]>([])
  const [version, setVersion] = useState('')
  useEffect(() => {
    void countryInfo(country).then((c) => setVersion(c?.version ?? ''))
  }, [country])
  const commit = (patch: Partial<Address>) => onCommit({ ...address, ...patch })
  // A place picked: its fields filled, AddAssist's version kept with them.
  const fill = (f: Found) => {
    setFound([])
    commit({ code: f.code || (address.code ?? ''), region: f.region, city: f.city, town: f.town, road: f.road, filledBy: version })
  }
  const search = async (query: string) => {
    if (!assisted || query.trim().length < 2) return setFound([])
    const result = await commands.addassistSearch(country, query, RESULTS)
    setFound(result.status === 'ok' ? result.data : [])
  }
  const codeDigits = country === 'jp' ? 7 : 5
  const onCode = async (code: string) => {
    const digits = code.replace(/\D/g, '').slice(0, codeDigits)
    commit({ code: digits })
    if (!assisted || digits.length !== codeDigits) return
    const result = await commands.addassistLookup(country, digits)
    const places = result.status === 'ok' ? result.data : []
    const [only, ...rest] = places
    if (only && rest.length === 0) fill(only)
    else setFound(places)
  }
  const text = (key: keyof Address, label: string, max = 60) => (
    <Field label={label}>
      <CommitInput
        value={String(address[key] ?? '')}
        placeholder={label}
        ariaLabel={label}
        maxLength={max}
        onCommit={(v) => commit({ [key]: v })}
      />
    </Field>
  )
  return (
    <Stack>
      <SegmentedToggle<'assisted' | 'hand'>
        value={assisted ? 'assisted' : 'hand'}
        options={[
          { value: 'assisted', label: t('addassist.assisted'), hint: t('addassist.assistedHint') },
          { value: 'hand', label: t('addassist.byHand') },
        ]}
        onChange={(v) => onAssisted(v === 'assisted')}
        ariaLabel={t('addassist.title')}
      />
      <Field label={t(`addassist.code.${country}`)}>
        <CommitInput
          value={address.code ?? ''}
          placeholder={t(`addassist.codePlaceholder.${country}`)}
          ariaLabel={t(`addassist.code.${country}`)}
          numeric
          maxLength={codeDigits + 1}
          onCommit={(v) => void onCode(v)}
        />
      </Field>
      {assisted && (
        <Field label={t('addassist.search')}>
          <CommitInput
            value=""
            placeholder={t(`addassist.searchPlaceholder.${country}`)}
            ariaLabel={t('addassist.search')}
            maxLength={64}
            liveMs={300}
            onCommit={(q) => void search(q)}
          />
        </Field>
      )}
      {found.length > 0 && (
        <RowList>
          {found.map((f) => (
            <ListRow key={`${f.code}${f.city}${f.town}${f.road}`} icon={<MapPin />} onClick={() => fill(f)}>
              {[f.code, f.region, f.city, f.town, f.road].filter(Boolean).join(' ')}
            </ListRow>
          ))}
        </RowList>
      )}
      {text('region', t(`addassist.region.${country}`))}
      {text('city', t(`addassist.city.${country}`))}
      {text('town', t(`addassist.town.${country}`))}
      {country === 'kr' && text('road', t('addassist.road'))}
      {text('line', t(`addassist.line.${country}`), 120)}
      {text('name', t('addassist.name'))}
      {country === 'kr' && (
        <SwitchField
          label={t('addassist.honorific')}
          checked={address.honorific ?? true}
          onChange={(honorific) => commit({ honorific })}
        />
      )}
      <SwitchField
        icon={<Languages />}
        label={t('addassist.latin')}
        checked={address.latin ?? false}
        onChange={(latin) => commit({ latin })}
      />
      {version && <Hint>{t('addassist.version', { version })}</Hint>}
    </Stack>
  )
}
