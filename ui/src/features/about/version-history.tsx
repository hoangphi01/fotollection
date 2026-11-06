// SPDX-License-Identifier: GPL-3.0-only
import { ChevronDown } from 'lucide-react'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import changelog from '../../../../CHANGELOG.md?raw'
import { FactLines, Field, Hint, RowList } from '@/components/forms/field'
import { Collapse } from '@/components/layout/collapse'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { showSettings } from '@/features/settings-window/use-settings-window'
import { releaseSummaries, type ReleaseSummary } from '@/lib/changelog'
import { cn } from '@/lib/utils'

/**
 * The app's version at the foot of every panel; clicking it opens the Settings window at
 * *What's new* (FR-086), where the versions, "Check for updates" and "Report a problem" live.
 */
export function VersionBadge({ version }: { version: string }) {
  const { t } = useTranslation()
  return (
    <button
      type="button"
      className="rounded-full"
      aria-label={t('about.versions')}
      data-hint={t('about.versions')}
      onClick={() => showSettings('whatsNew')}
    >
      <Badge variant="outline" className="cursor-pointer hover:border-primary hover:text-primary">
        v{version}
      </Badge>
    </button>
  )
}

/**
 * Every release with two lines on what it brought (FR-021), and *Show all* for its whole
 * entry, by kind (FR-086). The CHANGELOG is bundled at build time, so this works offline; the
 * version running is marked and opens unfolded. Entries are shown as text, never as HTML.
 */
export function ReleaseList({ version }: { version: string | undefined }) {
  const releases = useMemo(() => releaseSummaries(changelog), [])
  return (
    <RowList>
      {releases.map((r) => (
        <Release key={r.version} release={r} current={r.version === version} />
      ))}
    </RowList>
  )
}

function Release({ release: r, current }: { release: ReleaseSummary; current: boolean }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(current)
  return (
    <li>
      <Field>
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold tabular-nums">v{r.version}</span>
          {r.date && <span className="text-xs text-muted-foreground tabular-nums">{r.date}</span>}
          {current && (
            <Badge variant="outline" className="border-primary text-primary">
              {t('about.current')}
            </Badge>
          )}
          {r.notes.length > 0 && (
            <Button variant="ghost" size="xs" className="ml-auto" aria-expanded={open} onClick={() => setOpen(!open)}>
              <ChevronDown data-icon="inline-start" className={cn('transition-transform', open && 'rotate-180')} />
              {t(open ? 'about.showLess' : 'about.showAll')}
            </Button>
          )}
        </div>
        {r.lines.map((line) => (
          <Hint key={line}>{line}</Hint>
        ))}
        <Collapse open={open} stack="field">
          {r.notes.map((n) => (
            <Field key={n.kind} label={t(`about.kinds.${n.kind.toLowerCase()}`, { defaultValue: n.kind })}>
              <FactLines>
                {n.entries.map((entry) => (
                  <p key={entry} className="text-xs">
                    {entry}
                  </p>
                ))}
              </FactLines>
            </Field>
          ))}
        </Collapse>
      </Field>
    </li>
  )
}
