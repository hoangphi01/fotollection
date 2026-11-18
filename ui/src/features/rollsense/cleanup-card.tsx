// SPDX-License-Identifier: GPL-3.0-only
// @shown gallery: at the top of RollSense's side panel (STYLE_RULE §4).
import { Copy, Download, Eraser, Laugh, Smartphone } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { SectionCard } from '@/components/layout/section-card'
import type { CleanupKind, CleanupRow, SortPick } from '@/ipc/bindings'
import { formatBytes } from '@/lib/format'

const ICONS: Record<CleanupKind, typeof Copy> = { duplicates: Copy, screenshots: Smartphone, saved: Download, memes: Laugh }

/**
 * Clean up (FR-141, owner 2026-11-18, two UX reviews): what a roll's owner most often clears
 * out, by the task, each with its count and its bytes. A row shows its photos alone, to be
 * reviewed and marked to delete; nothing is ever ticked for the user.
 */
export function CleanupCard({ rows, picks, onPicks }: { rows: CleanupRow[]; picks: SortPick[]; onPicks(picks: SortPick[]): void }) {
  const { t, i18n } = useTranslation()
  if (rows.length === 0) return null
  const shown = JSON.stringify(picks)
  return (
    <SectionCard icon={<Eraser />} title={t('rollsense.cleanup')}>
      <ListRows>
        {rows.map((row) => {
          const Icon = ICONS[row.kind]
          return (
            <ListRow
              key={row.kind}
              icon={<Icon />}
              chosen={shown === JSON.stringify(row.picks)}
              trailing={
                <span className="text-xs text-muted-foreground tabular-nums">
                  {row.count.toLocaleString(i18n.language)} · {formatBytes(row.bytes ?? 0)}
                </span>
              }
              onClick={() => onPicks(row.picks)}
            >
              {t(`rollsense.cleanupKind.${row.kind}`)}
            </ListRow>
          )
        })}
      </ListRows>
    </SectionCard>
  )
}
