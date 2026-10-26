// SPDX-License-Identifier: GPL-3.0-only
import { Clock, RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import type { CameraClock } from '@/ipc/bindings'
import { formatOffset, parseOffset } from './clock-offset'
import { useCameraClocks } from './use-camera-clocks'

/** A body's name, with the end of its serial when two bodies share a model. */
function bodyName(clock: CameraClock, clocks: CameraClock[], unknown: string): string {
  const label = clock.label ?? unknown
  const twin = clocks.some((c) => c.id !== clock.id && c.label === clock.label)
  return twin && clock.serial ? `${label} · …${clock.serial.slice(-4)}` : label
}

/**
 * Camera clocks (FR-030): each body in the library with its correction, typed as
 * `+1:02:00`. The helper that reads the offset from two photos of one moment lives in the
 * preview's Captured card.
 */
export function CameraClocksCard() {
  const { t } = useTranslation()
  const { clocks, error, setOffset } = useCameraClocks()
  if (clocks.length === 0) return null
  return (
    <SectionCard icon={<Clock />} title={t('cameras.title')}>
      <p className="text-xs text-muted-foreground">{t('cameras.hint')}</p>
      <ul className="space-y-2">
        {clocks.map((clock) => {
          const offset = clock.offsetS ?? 0
          return (
            <li key={clock.id} className="flex items-center gap-2">
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm">{bodyName(clock, clocks, t('viewer.unknownCamera'))}</p>
                <p className="text-xs text-muted-foreground">
                  {t('cameras.photos', { count: clock.photos })}
                </p>
              </div>
              <div className="w-28">
                <CommitInput
                  value={offset === 0 ? '' : formatOffset(offset)}
                  placeholder={formatOffset(0)}
                  ariaLabel={t('cameras.offsetLabel')}
                  onCommit={(text) => {
                    const next = text.trim() === '' ? 0 : parseOffset(text)
                    if (next !== null) void setOffset(clock, next)
                  }}
                />
              </div>
              <Button
                size="icon-sm"
                variant="ghost"
                disabled={offset === 0}
                aria-label={t('cameras.reset')}
                data-hint={t('cameras.reset')}
                onClick={() => void setOffset(clock, 0)}
              >
                <RotateCcw />
              </Button>
            </li>
          )
        })}
      </ul>
      {error && <p className="text-xs text-destructive">{error}</p>}
    </SectionCard>
  )
}
