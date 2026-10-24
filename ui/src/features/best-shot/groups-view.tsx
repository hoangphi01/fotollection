// SPDX-License-Identifier: GPL-3.0-only
import { CheckCheck, LoaderCircle, Sparkles } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { CompareDialog } from './compare-dialog'
import { GroupStrip } from './group-strip'
import type { BestShotsApi } from './use-best-shots'

type GroupsViewProps = {
  bestShots: BestShotsApi
  thumbVersions: Record<number, number>
  generation: number
  revision: number
}

/** Groups of similar shots with the suggested best of each (FR-041). */
export function GroupsView({ bestShots, thumbVersions, generation, revision }: GroupsViewProps) {
  const { t } = useTranslation()
  const { groups, photosById, running, finished, analyze, keepBest, keepOne } = bestShots
  const [comparing, setComparing] = useState<number | null>(null)
  const clear = groups.filter((g) => !g.result.tooClose)
  const compared = groups.find((g) => g.id === comparing) ?? null

  if (groups.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center text-sm text-muted-foreground">
        {running ? (
          <LoaderCircle className="size-6 animate-spin" />
        ) : (
          <Sparkles className="size-6" />
        )}
        <p className="max-w-md">
          {running
            ? t('bestShot.running')
            : finished && !finished.cancelled
              ? t('bestShot.noneFound', { count: finished.analyzed })
              : t('bestShot.emptyBody')}
        </p>
        {!running && (
          <Button size="sm" onClick={() => void analyze()}>
            <Sparkles data-icon="inline-start" /> {t('bestShot.find')}
          </Button>
        )}
      </div>
    )
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 border-b px-4 py-2 text-xs text-muted-foreground">
        <span>
          {t('bestShot.summary', {
            count: groups.length,
            photos: groups.reduce((n, g) => n + g.result.photos.length, 0),
          })}
        </span>
        {finished && finished.failed > 0 && (
          <span className="text-destructive">
            {t('bestShot.failed', { count: finished.failed })}
          </span>
        )}
        <Button
          size="sm"
          variant="outline"
          className="ml-auto"
          disabled={clear.length === 0}
          onClick={() => void keepBest(clear.map((g) => g.id))}
        >
          <CheckCheck data-icon="inline-start" /> {t('bestShot.keepAll', { count: clear.length })}
        </Button>
      </div>
      <div className="flex-1 space-y-3 overflow-y-auto p-4">
        {groups.map((group) => (
          <GroupStrip
            key={group.id}
            group={group}
            photosById={photosById}
            thumbVersions={thumbVersions}
            generation={generation}
            onCompare={() => setComparing(group.id)}
            onKeepBest={() => void keepBest([group.id])}
          />
        ))}
      </div>
      <CompareDialog
        group={compared}
        photosById={photosById}
        revision={revision}
        onKeep={(g, p) => {
          void keepOne(g, p)
          setComparing(null)
        }}
        onClose={() => setComparing(null)}
      />
    </div>
  )
}
