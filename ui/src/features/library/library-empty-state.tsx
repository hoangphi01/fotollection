// SPDX-License-Identifier: GPL-3.0-only
import { FolderOpen, Images } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'

type LibraryEmptyStateProps = {
  onOpenFolder: () => void
}

export function LibraryEmptyState({ onOpenFolder }: LibraryEmptyStateProps) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <div className="flex size-14 items-center justify-center rounded-2xl bg-muted">
        <Images className="size-7 text-muted-foreground" />
      </div>
      <div className="space-y-1">
        <h1 className="font-heading text-lg font-semibold">{t('library.emptyTitle')}</h1>
        <p className="max-w-sm text-sm text-muted-foreground">{t('library.emptyBody')}</p>
      </div>
      <Button onClick={onOpenFolder}>
        <FolderOpen data-icon="inline-start" />
        {t('library.openFolder')}
      </Button>
    </div>
  )
}
