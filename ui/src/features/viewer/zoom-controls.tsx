// SPDX-License-Identifier: GPL-3.0-only
import { LoaderCircle, Scan, ZoomIn, ZoomOut } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'

type ZoomControlsProps = {
  percent: number
  zoomed: boolean
  canZoomIn: boolean
  /** Full detail for the zoomed view is on its way (FR-057). */
  loading: boolean
  /** The zoom shortcut's modifier as the platform names it. */
  mod: string
  onZoomIn: () => void
  onZoomOut: () => void
  onFit: () => void
}

/**
 * Zoom buttons for the preview (FR-057), so zoom can be found without knowing the
 * shortcuts or the trackpad pinch; each tooltip teaches its shortcut.
 */
export function ZoomControls({ percent, zoomed, canZoomIn, loading, mod, onZoomIn, onZoomOut, onFit }: ZoomControlsProps) {
  const { t } = useTranslation()
  const buttons = [
    { key: 'out', Icon: ZoomOut, label: t('viewer.zoomOut', { mod }), disabled: !zoomed, onClick: onZoomOut },
    { key: 'in', Icon: ZoomIn, label: t('viewer.zoomIn', { mod }), disabled: !canZoomIn, onClick: onZoomIn },
    { key: 'fit', Icon: Scan, label: t('viewer.zoomFit', { mod }), disabled: !zoomed, onClick: onFit },
  ] as const
  const button = ({ key, Icon, label, disabled, onClick }: (typeof buttons)[number]) => (
    <Tooltip key={key}>
      <TooltipTrigger asChild>
        <span>
          <Button variant="secondary" size="icon-sm" aria-label={label} disabled={disabled} onClick={onClick}>
            <Icon />
          </Button>
        </span>
      </TooltipTrigger>
      <TooltipContent>
        {label}
        {key === 'in' && <span className="block opacity-70">{t('viewer.zoomPinch')}</span>}
      </TooltipContent>
    </Tooltip>
  )
  return (
    <div className="absolute right-3 bottom-3 flex items-center gap-1 rounded-lg bg-black/60 p-1">
      {button(buttons[0])}
      <span
        className="flex min-w-14 items-center justify-center gap-1 px-1 text-xs text-white/80 tabular-nums"
        aria-live="polite"
        data-hint={loading ? t('viewer.zoomSharpening') : undefined}
      >
        {loading && <LoaderCircle className="size-3 animate-spin" aria-label={t('viewer.zoomSharpening')} />}
        {percent} %
      </span>
      {buttons.slice(1).map(button)}
    </div>
  )
}

