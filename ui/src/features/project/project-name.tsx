// SPDX-License-Identifier: GPL-3.0-only
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'

type ProjectNameProps = {
  name: string | null
  onRename(name: string): Promise<boolean>
}

/** The project's name; double-click to rename it in place (FR-023). */
export function ProjectName({ name, onRename }: ProjectNameProps) {
  const { t } = useTranslation()
  const shown = name ?? t('project.untitled')
  const [draft, setDraft] = useState<string | null>(null)

  const commit = async () => {
    if (draft === null) return
    const next = draft.trim()
    // Keep the box open on a refused name, so it can be corrected.
    if (next === '' || next === shown || (await onRename(next))) setDraft(null)
  }

  if (draft === null) {
    return (
      <span
        className="truncate"
        title={t('project.renameHint')}
        onDoubleClick={() => setDraft(shown)}
      >
        {shown}
      </span>
    )
  }
  return (
    <Input
      autoFocus
      value={draft}
      maxLength={120}
      aria-label={t('project.rename')}
      className="h-7 px-1.5 text-sm"
      onFocus={(e) => e.currentTarget.select()}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => void commit()}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          e.preventDefault()
          void commit()
        } else if (e.key === 'Escape') {
          e.preventDefault()
          setDraft(null)
        }
      }}
    />
  )
}
