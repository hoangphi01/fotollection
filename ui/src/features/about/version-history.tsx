// SPDX-License-Identifier: GPL-3.0-only
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import changelog from '../../../../CHANGELOG.md?raw'
import { Badge } from '@/components/ui/badge'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { releaseSummaries } from '@/lib/changelog'
import { cn } from '@/lib/utils'
import { UpdateCheck } from './update-check'

/**
 * The app's version; clicking it lists every release with two lines on what it brought.
 * The CHANGELOG is bundled at build time, so this works offline (FR-021); checking for a newer
 * version is a button, the only time the app goes online (FR-061).
 */
export function VersionHistory({ version }: { version: string }) {
  const { t } = useTranslation()
  const releases = useMemo(() => releaseSummaries(changelog), [])

  return (
    <Popover>
      <PopoverTrigger asChild>
        <button type="button" className="rounded-full" aria-label={t('about.versions')} data-hint={t('about.versions')}>
          <Badge variant="outline" className="cursor-pointer hover:border-primary hover:text-primary">
            v{version}
          </Badge>
        </button>
      </PopoverTrigger>
      <PopoverContent side="top" align="end" className="w-80 p-0">
        <p className="border-b px-4 py-2.5 text-sm font-semibold">{t('about.versions')}</p>
        <ol className="max-h-80 space-y-0 overflow-y-auto">
          {releases.map((r) => (
            <li key={r.version} className={cn('border-b px-4 py-2.5 last:border-b-0', r.version === version && 'bg-muted/60')}>
              <div className="flex items-baseline gap-2">
                <span className="text-sm font-semibold tabular-nums">v{r.version}</span>
                {r.date && <span className="text-xs text-muted-foreground tabular-nums">{r.date}</span>}
                {r.version === version && (
                  <Badge variant="outline" className="ml-auto border-primary text-primary">
                    {t('about.current')}
                  </Badge>
                )}
              </div>
              {r.lines.map((line) => (
                <p key={line} className="mt-0.5 line-clamp-1 text-xs text-muted-foreground">
                  {line}
                </p>
              ))}
            </li>
          ))}
        </ol>
        <UpdateCheck />
      </PopoverContent>
    </Popover>
  )
}
