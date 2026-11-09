// SPDX-License-Identifier: GPL-3.0-only
// @shown settings: in the Settings window only (STYLE_RULE §4, checked by check-lego).
import { ArrowRightLeft, Download, FolderOpen, HardDrive, RotateCcw, Upload } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RemoveButton } from '@/components/forms/danger-button'
import { FactLines, Field, ValueRow } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import { PanelNote } from '@/components/layout/panel-note'
import { SectionCard } from '@/components/layout/section-card'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { commands, type DataKind, type ImportMode, type SettingsImportSummary, type StoreUsage } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { formatBytes } from '@/lib/format'

/** What a red button of this section asks before it acts. */
type Asked = 'luts' | 'addresses' | 'reset' | 'erase'

/**
 * Settings → *Data* (FR-116, FR-082): moving this computer's settings to another one, what
 * Fotollection keeps here and where, and starting afresh. Rust owns every store and every
 * path; this section asks and shows. After an import, a reset or an erase the window reloads,
 * so every card reads the new settings.
 */
export function DataSection() {
  const { t } = useTranslation()
  const [usage, setUsage] = useState<StoreUsage[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [addresses, setAddresses] = useState(false)
  const [exported, setExported] = useState<string | null>(null)
  const [summary, setSummary] = useState<SettingsImportSummary | null>(null)
  const [mode, setMode] = useState<ImportMode>('merge')
  const [asked, setAsked] = useState<Asked | null>(null)
  const load = useCallback(() => {
    void commands.dataUsage().then((r) => (r.status === 'ok' ? setUsage(r.data) : setError(errorText(r.error))))
  }, [])
  useEffect(load, [load])
  const row = (kind: DataKind) => usage?.find((u) => u.kind === kind)
  const done = (r: { status: 'ok' } | { status: 'error'; error: Parameters<typeof errorText>[0] }) =>
    r.status === 'ok' ? window.location.reload() : setError(errorText(r.error))

  const exportSettings = async () => {
    const r = await commands.exportSettings(addresses)
    if (r.status === 'ok') setExported(r.data)
    else setError(errorText(r.error))
  }
  const inspect = async () => {
    const r = await commands.inspectSettingsImport()
    if (r.status === 'ok') setSummary(r.data)
    else setError(errorText(r.error))
  }
  const act = async (what: Asked) => {
    setAsked(null)
    if (what === 'luts') {
      await commands.removeUnusedLuts()
      load()
    } else if (what === 'addresses') {
      await commands.forgetAllAddresses()
      load()
    } else if (what === 'reset') done(await commands.resetSettings())
    else done(await commands.eraseData())
  }

  const kinds: DataKind[] = ['settings', 'looks', 'luts', 'fonts', 'logos', 'addresses', 'cameraSessions', 'cache', 'logs']
  return (
    <>
      {error && <PanelNote danger>{error}</PanelNote>}
      <SectionCard icon={<ArrowRightLeft />} title={t('data.move.title')}>
        <PanelNote icon={<ArrowRightLeft />}>{t('data.move.how')}</PanelNote>
        <SwitchField
          label={t('data.move.addresses')}
          hint={t('data.move.addressesHint')}
          checked={addresses}
          onChange={setAddresses}
        />
        <Field hint={exported ? t('data.move.exported', { path: exported }) : t('data.move.exportHint')}>
          <Button variant="outline" size="sm" onClick={() => void exportSettings()}>
            <Upload data-icon="inline-start" /> {t('data.move.export')}
          </Button>
        </Field>
        <Field hint={t('data.move.importHint')}>
          <Button variant="outline" size="sm" onClick={() => void inspect()}>
            <Download data-icon="inline-start" /> {t('data.move.import')}
          </Button>
        </Field>
      </SectionCard>

      <SectionCard icon={<HardDrive />} title={t('data.stores.title')}>
        {kinds.map((kind) => {
          const u = row(kind)
          return (
            <ValueRow key={kind} label={t(`data.stores.${kind}`)}>
              {/* Looks and addresses live inside the settings: their count alone. */}
              {u
                ? u.bytes
                  ? t('data.stores.size', { size: formatBytes(u.bytes), count: u.count })
                  : t('data.stores.items', { count: u.count })
                : t('cache.measuring')}
              <Button
                variant="outline"
                size="icon-sm"
                aria-label={t('data.stores.open')}
                data-hint={t('data.stores.open')}
                onClick={() => void commands.openDataFolder(kind).then((r) => r.status === 'error' && setError(errorText(r.error)))}
              >
                <FolderOpen />
              </Button>
            </ValueRow>
          )
        })}
        <Field hint={t('data.stores.unusedHint')}>
          <RemoveButton
            kind="delete"
            wide
            disabled={!row('luts')?.unused}
            label={t('data.stores.removeUnused', { count: row('luts')?.unused ?? 0 })}
            onClick={() => setAsked('luts')}
          />
        </Field>
        <RemoveButton
          kind="clear"
          wide
          disabled={!row('addresses')?.count}
          label={t('data.stores.forgetAddresses')}
          onClick={() => setAsked('addresses')}
        />
      </SectionCard>

      <SectionCard icon={<RotateCcw />} title={t('data.reset.title')}>
        <Field hint={t('data.reset.settingsHint')}>
          <RemoveButton kind="delete" wide label={t('data.reset.settings')} onClick={() => setAsked('reset')} />
        </Field>
        <Field hint={t('data.reset.eraseHint')}>
          <RemoveButton kind="delete" wide label={t('data.reset.erase')} onClick={() => setAsked('erase')} />
        </Field>
      </SectionCard>

      <AlertDialog open={asked !== null} onOpenChange={(open) => !open && setAsked(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{asked && t(`data.ask.${asked}.title`)}</AlertDialogTitle>
            <AlertDialogDescription>{asked && t(`data.ask.${asked}.body`)}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            {asked && <RemoveButton kind="delete" label={t(`data.ask.${asked}.do`)} onClick={() => void act(asked)} />}
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={summary !== null} onOpenChange={(open) => !open && setSummary(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('data.import.title')}</AlertDialogTitle>
            <AlertDialogDescription>{t('data.import.body')}</AlertDialogDescription>
          </AlertDialogHeader>
          {summary && (
            <FactLines>
              <span>{t('data.import.looks', { count: summary.looks })}</span>
              <span>{t('data.import.luts', { count: summary.luts })}</span>
              <span>{t('data.import.fonts', { count: summary.fonts })}</span>
              <span>{t('data.import.logos', { count: summary.logos })}</span>
              <span>{t('data.import.cameras', { count: summary.cameras })}</span>
              <span>{t('data.import.sizes', { count: summary.sizes })}</span>
              <span>{t('data.import.addresses', { count: summary.addresses })}</span>
            </FactLines>
          )}
          <Field hint={t(mode === 'merge' ? 'data.import.mergeHint' : 'data.import.replaceHint')}>
            <SegmentedToggle<ImportMode>
              value={mode}
              ariaLabel={t('data.import.mode')}
              options={[
                { value: 'merge', label: t('data.import.merge') },
                { value: 'replace', label: t('data.import.replace') },
              ]}
              onChange={setMode}
            />
          </Field>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void commands.applySettingsImport(mode).then(done)}>{t('data.import.do')}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
