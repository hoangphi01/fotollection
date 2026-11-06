// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { Crop, Download, Eye, PanelBottomClose, PanelBottomOpen, Redo2, RotateCcw, RotateCw, ShoppingBasket, Undo2 } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { KeysInfo } from '@/components/layout/keys-info'
import { Separator } from '@/components/ui/separator'
import { commands, type PhotoSummary } from '@/ipc/bindings'
import { MOD } from '@/lib/platform'

type PreviewToolbarProps = {
  photo: PhotoSummary | undefined
  /** Why the crop cannot be changed (the frame's Fit mode), or null when it can. */
  cropBlocked: string | null
  onCrop: () => void
  before: boolean
  onBefore: () => void
  canUndo: boolean
  canRedo: boolean
  onExport: () => void
  /** The zoom buttons, in the row (`ZoomControls inline`). */
  zoom: ReactNode
  index: number
  count: number
  filmstrip: boolean
  onFilmstrip: () => void
}

/**
 * The photo preview's row of tools, above the photo so nothing covers it (FR-095): the photo's
 * own actions first, then the view and history, then zoom, where in the set, the filmstrip and
 * every key. Each hint names its key, as Lightroom's do, so a user learns them on the way.
 */
export function PreviewToolbar({
  photo,
  cropBlocked,
  onCrop,
  before,
  onBefore,
  canUndo,
  canRedo,
  onExport,
  zoom,
  index,
  count,
  filmstrip,
  onFilmstrip,
}: PreviewToolbarProps) {
  const { t } = useTranslation()
  const inBasket = photo?.queue === 'print'
  const rotate = (clockwise: boolean) => photo && void commands.rotatePhotos([photo.id], clockwise)
  const stripLabel = t(filmstrip ? 'viewer.hideFilmstrip' : 'viewer.showFilmstrip')
  return (
    <div className="@container flex h-11 shrink-0 items-center gap-1.5 border-b bg-background px-3">
      {/* The hint sits on the wrapper, as a disabled button gets no pointer events. */}
      <span data-hint={cropBlocked ?? t('viewer.cropHint')}>
        <ToolbarButton icon={<Crop />} label={t('viewer.crop')} disabled={!photo || cropBlocked !== null} onClick={onCrop} />
      </span>
      <ToolbarButton
        icon={<RotateCcw />}
        label={t('viewer.rotateLeftShort')}
        data-hint={t('viewer.rotateLeft', { mod: MOD })}
        iconOnly
        disabled={!photo}
        onClick={() => rotate(false)}
      />
      <ToolbarButton
        icon={<RotateCw />}
        label={t('viewer.rotateRightShort')}
        data-hint={t('viewer.rotateRight', { mod: MOD })}
        iconOnly
        disabled={!photo}
        onClick={() => rotate(true)}
      />
      {photo?.queue === 'aside' ? (
        <ToolbarButton
          icon={<Undo2 />}
          label={t('select.toGallery')}
          data-hint={t('preview.toGalleryHint')}
          onClick={() => void commands.setPhotoQueue([photo.id], null)}
        />
      ) : (
        <ToolbarButton
          icon={<ShoppingBasket />}
          label={t(inBasket ? 'preview.inBasket' : 'preview.toBasket')}
          variant={inBasket ? 'default' : 'outline'}
          aria-pressed={inBasket}
          data-hint={t('preview.basketHint')}
          disabled={!photo}
          onClick={() => photo && void commands.setPhotoQueue([photo.id], inBasket ? null : 'print')}
        />
      )}
      <Separator orientation="vertical" className="mx-1 h-5!" />
      {/* Before / After (FR-079, FR-096): the photo without its colour, on or off. */}
      <ToolbarButton
        icon={<Eye />}
        label={t('look.before')}
        variant={before ? 'default' : 'outline'}
        aria-pressed={before}
        data-hint={t('look.beforeHint')}
        disabled={!photo}
        onClick={onBefore}
      />
      <Separator orientation="vertical" className="mx-1 h-5!" />
      <ToolbarButton
        icon={<Undo2 />}
        label={t('viewer.undo')}
        data-hint={t('viewer.undoHint', { mod: MOD })}
        iconOnly
        disabled={!canUndo}
        onClick={() => void commands.undo()}
      />
      <ToolbarButton
        icon={<Redo2 />}
        label={t('viewer.redo')}
        data-hint={t('viewer.redoHint', { mod: MOD })}
        iconOnly
        disabled={!canRedo}
        onClick={() => void commands.redo()}
      />
      <Separator orientation="vertical" className="mx-1 h-5!" />
      <ToolbarButton
        icon={<Download />}
        label={t('export.title')}
        data-hint={t('viewer.exportHint', { mod: MOD })}
        disabled={!photo}
        onClick={onExport}
      />
      <div className="ml-auto flex items-center gap-1.5">
        {zoom}
        <span className="min-w-16 text-center text-xs text-muted-foreground tabular-nums">
          {index + 1} / {count}
        </span>
        <ToolbarButton
          icon={filmstrip ? <PanelBottomClose /> : <PanelBottomOpen />}
          label={stripLabel}
          variant={filmstrip ? 'default' : 'outline'}
          aria-pressed={filmstrip}
          data-hint={stripLabel}
          iconOnly
          onClick={onFilmstrip}
        />
        <KeysInfo lines={t('viewer.keys', { mod: MOD }).split('\n')} />
      </div>
    </div>
  )
}
