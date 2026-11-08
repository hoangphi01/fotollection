// SPDX-License-Identifier: GPL-3.0-only
import { Scan, ZoomIn, ZoomOut } from 'lucide-react'
import { Spinner } from '@/components/forms/spinner'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

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
  /** In a toolbar row (the photo preview, FR-095) rather than over the stage: the percentage
   * alone, the photo's own (1:1 is 100 %), a click on it going fit ↔ 1:1; the buttons hide when
   * the row is narrow (`@xl`), never a word that could outgrow its box (LESSONS 2026-11-06). */
  inline?: boolean
  /** Fit ↔ 1:1, for a click on the percentage in a toolbar row. */
  onToggle?: () => void
}

/**
 * Zoom buttons for the preview (FR-057), so zoom can be found without knowing the
 * shortcuts or the trackpad pinch; each tooltip teaches its shortcut.
 */
export function ZoomControls({ percent, zoomed, canZoomIn, loading, mod, onZoomIn, onZoomOut, onFit, inline = false, onToggle }: ZoomControlsProps) {
  const { t } = useTranslation()
  const buttons = [
    { key: 'out', Icon: ZoomOut, label: t('viewer.zoomOut', { mod }), disabled: !zoomed, onClick: onZoomOut },
    { key: 'in', Icon: ZoomIn, label: t('viewer.zoomIn', { mod }), disabled: !canZoomIn, onClick: onZoomIn },
    { key: 'fit', Icon: Scan, label: t('viewer.zoomFit', { mod }), disabled: !zoomed, onClick: onFit },
  ] as const
  const hint = (key: string, label: string) => (key === 'in' ? `${label}\n${t('viewer.zoomPinch')}` : label)
  const button = ({ key, Icon, label, disabled, onClick }: (typeof buttons)[number]) =>
    inline ? (
      // In a toolbar, its buttons are the toolbar's own (STYLE_RULE §4), hidden when narrow.
      <span key={key} className="hidden @xl:block">
        <ToolbarButton icon={<Icon />} label={label} data-hint={hint(key, label)} iconOnly disabled={disabled} onClick={onClick} />
      </span>
    ) : (
      // On the stage's chip; the hint on a wrapper, as a disabled button gets no pointer events.
      <span key={key} data-hint={hint(key, label)}>
        <Button variant="secondary" size="icon-sm" aria-label={label} disabled={disabled} onClick={onClick}>
          <Icon />
        </Button>
      </span>
    )
  return (
    <div className={cn('flex items-center', inline ? 'gap-2' : 'absolute right-3 bottom-3 gap-1 rounded-lg bg-stage-chip p-1')}>
      {button(buttons[0])}
      {inline ? (
        <Button
          variant="outline"
          size="sm"
          aria-live="polite"
          data-hint={loading ? t('viewer.zoomSharpening') : t('viewer.zoomToggle')}
          className="min-w-14 justify-center px-1 text-xs font-normal text-muted-foreground tabular-nums"
          onClick={onToggle}
        >
          {loading && <Spinner aria-label={t('viewer.zoomSharpening')} />}
          {percent}&nbsp;%
        </Button>
      ) : (
        <span
          className="flex min-w-14 items-center justify-center gap-1 px-1 text-xs text-stage-ink tabular-nums"
          aria-live="polite"
          data-hint={loading ? t('viewer.zoomSharpening') : undefined}
        >
          {loading && <Spinner className="size-3" aria-label={t('viewer.zoomSharpening')} />}
          {percent}&nbsp;%
        </span>
      )}
      {buttons.slice(1).map(button)}
    </div>
  )
}

