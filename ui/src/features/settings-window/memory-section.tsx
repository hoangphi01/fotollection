// SPDX-License-Identifier: GPL-3.0-only
// @shown settings: in the Settings window only (STYLE_RULE §4, checked by check-lego).
import { Cpu, HardDrive, Sparkles, Wand2 } from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FactLines, Field, Hint, ValueRow } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import { GpuLook } from '@/features/viewer/gpu-look'
import { commands, type CacheUsage, type MachineInfo, type MemoryInfo } from '@/ipc/bindings'
import { formatBytes, formatMemory } from '@/lib/format'
import { RemoveButton, ResetButton } from '@/components/forms/danger-button'

/** The shares of memory offered (Rust's `PREVIEW_MEMORY_CHOICES`) and the cache limits. */
const MEMORY = ['15', '25', '40'] as const
const LIMITS = ['2', '5', '10', '20'] as const
/** How often the memory held is measured again while the section is open. */
const REFRESH_MS = 2000

/**
 * The Settings window's *RAM & cache* section (FR-086, FR-091, ADR-0041): the share of the
 * computer's memory the previews keep, the disk cache's limit and what it holds, whether
 * previews are prepared in the background, and *Default* and *Auto*. Each control says what
 * it does to speed, with the measured numbers (RESULT §29).
 */
export function MemorySection() {
  const { t } = useTranslation()
  const { ui, update } = useUiSettings()
  const [memory, setMemory] = useState<MemoryInfo | null>(null)
  const [cache, setCache] = useState<CacheUsage | null>(null)
  const [clearing, setClearing] = useState(false)
  const [machine, setMachine] = useState<MachineInfo | null>(null)
  // The graphics card as the window sees it, and whether the colour sliders use it (ADR-0039).
  const gpu = useMemo(() => ({ name: GpuLook.renderer(), used: GpuLook.worthIt() }), [])
  useEffect(() => {
    let live = true
    void commands.machineInfo().then((m) => live && setMachine(m))
    return () => {
      live = false
    }
  }, [])

  useEffect(() => {
    let live = true
    const measure = () => void commands.memoryInfo().then((m) => live && setMemory(m))
    measure()
    const timer = window.setInterval(measure, REFRESH_MS)
    return () => {
      live = false
      window.clearInterval(timer)
    }
  }, [ui.previewMemoryPercent])

  const measureCache = useCallback(() => {
    void commands.cacheUsage().then(setCache)
  }, [])
  useEffect(measureCache, [measureCache, ui.cacheLimitGb])

  const clear = async () => {
    setClearing(true)
    setCache(await commands.clearCache())
    setClearing(false)
  }
  const suggest = async (auto: boolean) => {
    const s = await commands.suggestMemorySettings(auto)
    await update({
      previewMemoryPercent: s.previewMemoryPercent,
      cacheLimitGb: s.cacheLimitGb,
      preparePreviews: s.preparePreviews,
    })
  }

  const share = (percent: number) => (memory ? formatMemory(((memory.total ?? 0) * percent) / 100) : '')
  const memoryOptions = MEMORY.map((p) => ({
    value: p,
    label: `${p} %`,
    hint: share(Number(p)),
  }))
  const limitOptions = LIMITS.map((g) => ({ value: g, label: `${g} GB` }))

  return (
    <>
      <SectionCard icon={<Cpu />} title={t('memory.title')}>
        <Field label={t('memory.share')} hint={t('memory.hint')}>
          <SegmentedToggle
            value={String(ui.previewMemoryPercent)}
            options={memoryOptions}
            onChange={(p) => void update({ previewMemoryPercent: Number(p) })}
            ariaLabel={t('memory.share')}
          />
        </Field>
        {memory && (
          <ValueRow label={t('memory.held')}>
            {t('memory.heldValue', {
              held: formatMemory(memory.held ?? 0),
              budget: formatMemory(memory.budget ?? 0),
              total: formatMemory(memory.total ?? 0),
            })}
          </ValueRow>
        )}
      </SectionCard>

      <SectionCard icon={<HardDrive />} title={t('cache.title')}>
        <Field label={t('cache.limit')} hint={t('cache.limitHint')}>
          <SegmentedToggle
            value={String(ui.cacheLimitGb)}
            options={limitOptions}
            onChange={(g) => void update({ cacheLimitGb: Number(g) })}
            ariaLabel={t('cache.limit')}
          />
        </Field>
        <FactLines>
          <ValueRow label={t('cache.thumbnails')}>{cache ? formatBytes(cache.thumbnailsBytes ?? 0) : t('cache.measuring')}</ValueRow>
          <ValueRow label={t('cache.previews')}>{cache ? formatBytes(cache.previewsBytes ?? 0) : ''}</ValueRow>
          <ValueRow label={t('cache.analyses')}>{cache ? formatBytes(cache.analysesBytes ?? 0) : ''}</ValueRow>
          <ValueRow label={t('cache.total')}>
            {cache
              ? t('cache.totalValue', {
                  size: formatBytes(cache.bytes ?? 0),
                  limit: formatBytes(cache.limitBytes ?? 0),
                })
              : ''}
          </ValueRow>
        </FactLines>
        <SwitchField
          icon={<Sparkles />}
          label={t('cache.prepare')}
          checked={ui.preparePreviews}
          onChange={(on) => void update({ preparePreviews: on })}
          hint={t('cache.prepareHint')}
        />
        <Field hint={t('cache.note')}>
          <RemoveButton
            kind="delete"
            wide
            disabled={!cache || cache.files === 0 || clearing}
            label={t('cache.clear')}
            hint={t('cache.clearHint')}
            onClick={() => void clear()}
          />
        </Field>
      </SectionCard>

      <SectionCard icon={<Wand2 />} title={t('memory.forThisComputer')}>
        {machine && (
          <FactLines>
            <ValueRow label={t('machine.cpu')}>
              {t('machine.cpuValue', { name: machine.cpu ?? t('machine.unknown'), cores: machine.cores })}
            </ValueRow>
            <ValueRow label={t('machine.gpu')}>
              {t(gpu.used ? 'machine.gpuUsed' : 'machine.gpuNotUsed', { name: gpu.name ?? t('machine.unknown') })}
            </ValueRow>
            <ValueRow label={t('machine.memory')}>{formatMemory(machine.memory ?? 0)}</ValueRow>
            <ValueRow label={t('machine.disk')}>
              {machine.freeDisk == null ? t('machine.unknown') : t('machine.diskValue', { free: formatBytes(machine.freeDisk) })}
            </ValueRow>
          </FactLines>
        )}
        <Field
          hint={t('memory.now', {
            percent: ui.previewMemoryPercent,
            limit: ui.cacheLimitGb,
            previews: t(ui.preparePreviews ? 'memory.prepared' : 'memory.notPrepared'),
          })}
        >
          <div className="grid grid-cols-2 gap-2">
            <ResetButton label={t('memory.default')} hint={t('memory.defaultHint')} onClick={() => void suggest(false)} />
            <Button size="sm" onClick={() => void suggest(true)} data-hint={t('memory.autoHint')}>
              <Wand2 data-icon="inline-start" /> {t('memory.auto')}
            </Button>
          </div>
        </Field>
        <Hint>{t('memory.autoExplain')}</Hint>
      </SectionCard>
    </>
  )
}
