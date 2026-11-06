// SPDX-License-Identifier: GPL-3.0-only
import { Info } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'

/**
 * The keyboard and mouse hints of a toolbar behind an ⓘ button: shown on click, hidden
 * otherwise, so the toolbar stays short. The same in the gallery and in Collage mode.
 */
export function KeysInfo({ lines }: { lines: string[] }) {
  const { t } = useTranslation()
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="outline" size="icon-sm" aria-label={t('app.keysInfo')} data-hint={t('app.keysInfo')}>
          <Info />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-80 space-y-1.5 text-xs text-muted-foreground">
        {lines.map((line) => (
          <p key={line}>{line}</p>
        ))}
      </PopoverContent>
    </Popover>
  )
}
