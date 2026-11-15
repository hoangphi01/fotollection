// SPDX-License-Identifier: GPL-3.0-only
import { Languages, MapPin } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands, type Address, type Country, type CountryInfo, type Found } from '@/ipc/bindings'
import { emptyAddress } from '@/lib/address'
import { CommitInput } from './commit-input'
import { Badge } from '@/components/ui/badge'
import { FactLines, Field, FieldLabel, Hint, Inset, Stack } from './field'
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

/** An address's places on one line, as the post's data names them. */
const places = (a: { region?: string; city?: string; town?: string; road?: string }) =>
  [a.region, a.city, a.town, a.road].filter(Boolean).join(' ')

/** A code written as the country writes it: 〒271-0092, 03172. */
const codeLabel = (country: Country, code: string) =>
  country === 'jp' && code.length === 7 ? `〒${code.slice(0, 3)}-${code.slice(3)}` : code

type AddAssistProps = {
  /** The block's title (Recipient), its AddAssist badge in the corner (the style board). */
  label?: string
  country: Country
  value: Address | null
  /** Called with the whole address when a field is left or a place picked (a draft never). */
  onCommit: (address: Address) => void
  /** By hand or assisted, one switch for every address field of the machine. */
  assisted: boolean
  onAssisted: (assisted: boolean) => void
}

/**
 * AddAssist (FR-133), the address field of every Japanese or Korean card. Assisted (owner,
 * 2026-11-15): one field takes the postal code or a place's name, a small box shows the address
 * found (or the places to pick from), and only the block and building number and the name are
 * typed; the post's data is looked up offline in Rust. By hand: every line typed. The one block
 * for an address: `check-lego` refuses AddAssist's commands anywhere else.
 */
export function AddAssist({ label, country, value, onCommit, assisted, onAssisted }: AddAssistProps) {
  const { t } = useTranslation()
  const address = value ?? emptyAddress(country)
  const [found, setFound] = useState<Found[] | null>(null)
  const [version, setVersion] = useState('')
  useEffect(() => {
    void countryInfo(country).then((c) => setVersion(c?.version ?? ''))
  }, [country])
  const commit = (patch: Partial<Address>) => onCommit({ ...address, ...patch })
  // A place picked: its fields filled, AddAssist's version kept with them.
  const fill = (f: Found) => {
    setFound(null)
    commit({ code: f.code || (address.code ?? ''), region: f.region, city: f.city, town: f.town, road: f.road, filledBy: version })
  }
  const codeDigits = country === 'jp' ? 7 : 5
  // The code as the country writes it (owner, 2026-11-15): Japan's in two parts, 123 – 4567,
  // the second taking the cursor once the first is full; Korea's five digits in one.
  const head = country === 'jp' ? 3 : 5
  const [parts, setParts] = useState<[string, string]>(() => {
    const code = address.code ?? ''
    return [code.slice(0, head), code.slice(head)]
  })
  const codeRow = useRef<HTMLDivElement>(null)
  const lookUpCode = async (digits: string) => {
    if (digits.length !== codeDigits) return setFound(null)
    const result = await commands.addassistLookup(country, digits)
    const hits = result.status === 'ok' ? result.data : []
    const [only, ...rest] = hits
    if (only && rest.length === 0) return fill(only)
    setFound(hits)
  }
  const onPart = (i: 0 | 1, value: string) => {
    const digits = value.replace(/\D/g, '')
    const next: [string, string] = i === 0 ? [digits, parts[1]] : [parts[0], digits]
    setParts(next)
    if (i === 0 && digits.length === head && country === 'jp') codeRow.current?.querySelectorAll('input')[1]?.focus()
    void lookUpCode(next.join(''))
  }
  // A place by name, for a code not known: matsudo, まつど, 松戸; 세종대로 175.
  const search = async (query: string) => {
    if (query.trim().length < 2) return setFound(null)
    const result = await commands.addassistSearch(country, query, RESULTS)
    setFound(result.status === 'ok' ? result.data : [])
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
  const filled = places(address)
  return (
    <Stack>
      <div className="flex items-center justify-between gap-2">
        {label && <FieldLabel>{label}</FieldLabel>}
        {version && (
          <Badge variant="outline" className="ml-auto">
            {t('addassist.version', { version })}
          </Badge>
        )}
      </div>
      <SegmentedToggle<'assisted' | 'hand'>
        value={assisted ? 'assisted' : 'hand'}
        options={[
          { value: 'assisted', label: t('addassist.assisted'), hint: t('addassist.assistedHint') },
          { value: 'hand', label: t('addassist.byHand') },
        ]}
        onChange={(v) => onAssisted(v === 'assisted')}
        ariaLabel={t('addassist.title')}
      />
      {assisted ? (
        <>
          <Field label={t(`addassist.code.${country}`)} hint={t('addassist.enterHint')}>
            <div ref={codeRow} className="flex items-center gap-2">
              {country === 'jp' && (
                <span aria-hidden className="font-bold text-vermilion">
                  〒
                </span>
              )}
              <CommitInput
                value={parts[0]}
                placeholder={country === 'jp' ? '123' : '12345'}
                ariaLabel={t(`addassist.code.${country}`)}
                numeric
                maxLength={head}
                vermilion={country === 'jp' ? 'heavy' : undefined}
                liveMs={country === 'jp' ? 50 : undefined}
                onCommit={(v) => onPart(0, v)}
              />
              {country === 'jp' && (
                <>
                  <span aria-hidden className="font-bold text-vermilion">
                    –
                  </span>
                  <CommitInput
                    value={parts[1]}
                    placeholder="4567"
                    ariaLabel={t('addassist.codeSecond')}
                    numeric
                    maxLength={codeDigits - head}
                    vermilion="light"
                    onCommit={(v) => onPart(1, v)}
                  />
                </>
              )}
            </div>
          </Field>
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
          {/* The address found, or the places to pick from (a code shared by several). */}
          {found && found.length > 0 ? (
            <Inset>
              {found.map((f) => (
                <ListRow key={`${f.code}${f.city}${f.town}${f.road}`} icon={<MapPin />} onClick={() => fill(f)}>
                  {[codeLabel(country, f.code), places(f)].filter(Boolean).join(' ')}
                </ListRow>
              ))}
            </Inset>
          ) : found ? (
            <Hint>{t('addassist.none')}</Hint>
          ) : (
            (filled || address.code) && (
              <Inset>
                <FactLines>
                  <Hint>{codeLabel(country, address.code ?? '')}</Hint>
                  <span className="text-sm">{filled}</span>
                </FactLines>
              </Inset>
            )
          )}
        </>
      ) : (
        <>
          <Field label={t(`addassist.code.${country}`)}>
            <CommitInput
              value={address.code ?? ''}
              placeholder={t(`addassist.codePlaceholder.${country}`)}
              ariaLabel={t(`addassist.code.${country}`)}
              numeric
              maxLength={codeDigits + 1}
              vermilion={country === 'jp' ? 'heavy' : undefined}
              onCommit={(v) => commit({ code: v.replace(/\D/g, '').slice(0, codeDigits) })}
            />
          </Field>
          {text('region', t(`addassist.region.${country}`))}
          {text('city', t(`addassist.city.${country}`))}
          {text('town', t(`addassist.town.${country}`))}
          {country === 'kr' && text('road', t('addassist.road'))}
        </>
      )}
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
        hint={t('addassist.latinHint')}
      />
    </Stack>
  )
}
