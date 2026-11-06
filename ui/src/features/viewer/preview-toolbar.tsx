// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import { ChevronDown, Columns2, Download, Eye, Maximize2, Minimize2, PanelBottomClose, PanelBottomOpen, Redo2, RotateCcw, RotateCw, Rows2, ShoppingBasket, SquareSplitHorizontal, SquareSplitVertical, Undo2 } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { KeysInfo } from '@/components/layout/keys-info'
import { Kbd } from '@/components/ui/kbd'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Separator } from '@/components/ui/separator'
import { commands, type PhotoSummary } from '@/ipc/bindings'
import { MOD, isMac } from '@/lib/platform'
import type { Compare, CompareMode } from './compare-stage'
import { ListRow } from '@/components/forms/list-row'

type PreviewToolbarProps = {
  photo: PhotoSummary | undefined
  before: boolean
  onBefore: () => void
  /** Lightroom's before / after views (FR-096). */
  compare: Compare
  onCompare: (mode: CompareMode) => void
  canUndo: boolean
  canRedo: boolean
  onExport: () => void
  /** The zoom buttons, in the row (`ZoomControls inline`). */
  zoom: ReactNode
  filmstrip: boolean
  onFilmstrip: () => void
  /** The preview fills the app window (FR-095). */
  windowed: boolean
  onLayout: () => void
}

/**
 * The photo preview's row of tools, above the photo so nothing covers it (FR-095): the photo's
 * own actions first, then the view and history, then zoom, where in the set, the filmstrip and
 * every key. Each hint names its key, as Lightroom's do, so a user learns them on the way.
 */
export function PreviewToolbar({
  photo,
  before,
  onBefore,
  compare,
  onCompare,
  canUndo,
  canRedo,
  onExport,
  zoom,
  filmstrip,
  onFilmstrip,
  windowed,
  onLayout,
}: PreviewToolbarProps) {
  const { t } = useTranslation()
  const inBasket = photo?.queue === 'print'
  const rotate = (clockwise: boolean) => photo && void commands.rotatePhotos([photo.id], clockwise)
  const stripLabel = t(filmstrip ? 'viewer.hideFilmstrip' : 'viewer.showFilmstrip')
  return (
    <div className="@container flex h-header items-center gap-1.5 px-3">
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
        variant={before || compare !== 'off' ? 'default' : 'outline'}
        aria-pressed={before}
        data-hint={t('look.beforeHint')}
        disabled={!photo}
        onClick={onBefore}
      />
      <CompareMenu compare={compare} before={before} onBefore={onBefore} onCompare={onCompare} disabled={!photo} />
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
        <ToolbarButton
          icon={filmstrip ? <PanelBottomClose /> : <PanelBottomOpen />}
          label={stripLabel}
          variant={filmstrip ? 'default' : 'outline'}
          aria-pressed={filmstrip}
          data-hint={stripLabel}
          iconOnly
          onClick={onFilmstrip}
        />
        <ToolbarButton
          icon={windowed ? <Minimize2 /> : <Maximize2 />}
          label={t(windowed ? 'viewer.layoutDialog' : 'viewer.layoutWindow')}
          variant={windowed ? 'default' : 'outline'}
          aria-pressed={windowed}
          data-hint={t(windowed ? 'viewer.layoutDialog' : 'viewer.layoutWindow')}
          iconOnly
          onClick={onLayout}
        />
        <KeysInfo lines={t('viewer.keys', { mod: MOD }).split('\n')} />
      </div>
    </div>
  )
}

const ALT = isMac ? '⌥' : 'Alt'

/** The views of *Before*, as Lightroom's menu beside its [Y|Y] button (FR-096). */
function CompareMenu({
  compare,
  before,
  onBefore,
  onCompare,
  disabled,
}: {
  compare: Compare
  before: boolean
  onBefore: () => void
  onCompare: (mode: CompareMode) => void
  disabled: boolean
}) {
  const { t } = useTranslation()
  const views = [
    { key: 'beforeOnly', icon: <Eye />, keys: ['\\'], on: before, pick: onBefore },
    { key: 'sideBySide', icon: <Columns2 />, keys: ['Y'], on: compare === 'sideBySide', pick: () => onCompare('sideBySide') },
    { key: 'split', icon: <SquareSplitHorizontal />, keys: ['⇧', 'Y'], on: compare === 'split', pick: () => onCompare('split') },
    { key: 'topBottom', icon: <Rows2 />, keys: [ALT, 'Y'], on: compare === 'topBottom', pick: () => onCompare('topBottom') },
    { key: 'splitTopBottom', icon: <SquareSplitVertical />, keys: [], on: compare === 'splitTopBottom', pick: () => onCompare('splitTopBottom') },
  ] as const
  return (
    <Popover>
      <PopoverTrigger asChild>
        <ToolbarButton icon={<ChevronDown />} label={t('compare.views')} data-hint={t('compare.views')} iconOnly disabled={disabled} />
      </PopoverTrigger>
      <PopoverContent align="start" className="w-64 p-1">
        {views.map((v) => (
          <ListRow
            key={v.key}
            icon={v.icon}
            chosen={v.on}
            trailing={v.keys.map((k) => (
              <Kbd key={k}>{k}</Kbd>
            ))}
            onClick={v.pick}
          >
            {t(`compare.${v.key}`)}
          </ListRow>
        ))}
      </PopoverContent>
    </Popover>
  )
}
