// SPDX-License-Identifier: GPL-3.0-only
import { HardDrive, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { commands, type CacheUsage as Usage } from '@/ipc/bindings'
import { formatBytes } from '@/lib/format'

/**
 * The disk cache (DATABASE.md §8): its size against the limit it is kept under, and *Clear
 * cache*. Nothing of the user's is lost: thumbnails and analyses are made again when needed.
 */
export function CacheUsage() {
  const { t } = useTranslation()
  const [usage, setUsage] = useState<Usage | null>(null)
  const [clearing, setClearing] = useState(false)

  useEffect(() => {
    let live = true
    void commands.cacheUsage().then((u) => live && setUsage(u))
    return () => {
      live = false
    }
  }, [])

  const clear = async () => {
    setClearing(true)
    setUsage(await commands.clearCache())
    setClearing(false)
  }

  return (
    <div className="space-y-2 border-t px-4 py-3">
      <div className="flex items-center gap-2 text-xs">
        <HardDrive className="size-3.5 text-muted-foreground" />
        <span className="tabular-nums">
          {usage
            ? t('cache.usage', { size: formatBytes(usage.bytes ?? 0), limit: formatBytes(usage.limitBytes ?? 0) })
            : t('cache.measuring')}
        </span>
        <Button
          size="xs"
          variant="ghost"
          className="ml-auto"
          disabled={!usage || usage.files === 0 || clearing}
          onClick={() => void clear()}
          data-hint={t('cache.clearHint')}
        >
          <Trash2 data-icon="inline-start" /> {t('cache.clear')}
        </Button>
      </div>
      <p className="text-xs text-muted-foreground">{t('cache.note')}</p>
    </div>
  )
}
