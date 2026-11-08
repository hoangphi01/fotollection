// SPDX-License-Identifier: GPL-3.0-only
// @shown preview
import {
  ArchiveRestore,
  ChevronDown,
  Columns2,
  Download,
  Eye,
  Maximize2,
  Minimize2,
  PanelBottomClose,
  PanelBottomOpen,
  PanelLeftClose,
  PanelLeftOpen,
  RotateCcwSquare,
  RotateCwSquare,
  Rows2,
  ShoppingBasket,
  SquareSplitHorizontal,
  SquareSplitVertical,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { ButtonGroup } from '@/components/forms/button-group'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { KeysInfo } from '@/components/layout/keys-info'
import { Kbd } from '@/components/ui/kbd'
import { Popover, PopoverTrigger } from '@/components/ui/popover'
import { Separator } from '@/components/ui/separator'
import { commands, type PhotoId, type PhotoSummary } from '@/ipc/bindings'
import { MOD, isMac } from '@/lib/platform'
import type { Compare, CompareMode } from './compare-stage'
import { ListRow } from '@/components/forms/list-row'
import { SettingsButtons } from '@/features/develop/settings-buttons'
import { HistoryButtons } from '@/features/history/history-buttons'
import { MenuContent } from '@/components/forms/menu-content'

type PreviewToolbarProps = {
  photo: PhotoSummary | undefined
  before: boolean
  onBefore: () => void
  /** Lightroom's before / after views (FR-096). */
  compare: Compare
  onCompare: (mode: CompareMode) => void
  canUndo: boolean
  canRedo: boolean
  /** Photos ticked in the gallery: *Paste settings* offers them (FR-097). */
  ticked: readonly PhotoId[]
  onExport: () => void
  /** The zoom buttons, in the row (`ZoomControls inline`). */
  zoom: ReactNode
  filmstrip: boolean
  onFilmstrip: () => void
  /** The preview fills the app window (FR-095). */
  windowed: boolean
  onLayout: () => void
  /** The left column a narrow window folds (FR-095): shown or hidden from here, as Tab does. */
  side?: { open: boolean; onToggle: () => void }
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
  ticked,
  onExport,
  zoom,
  filmstrip,
  onFilmstrip,
  windowed,
  onLayout,
  side,
}: PreviewToolbarProps) {
  const { t } = useTranslation()
  const inBasket = photo?.queue === 'print'
  const rotate = (clockwise: boolean) => photo && void commands.rotatePhotos([photo.id], clockwise)
  const stripLabel = t(filmstrip ? 'viewer.hideFilmstrip' : 'viewer.showFilmstrip')
  return (
    // Too narrow for every button (a narrow window with both columns open), the row scrolls
    // sideways rather than cut its last buttons (owner, 2026-11-07).
    <div className="@container flex h-header items-center gap-2 overflow-x-auto overflow-y-hidden px-3 [scrollbar-width:thin] [&>*]:shrink-0">
      {side && (
        <>
          <ToolbarButton
            icon={side.open ? <PanelLeftClose /> : <PanelLeftOpen />}
            label={t(side.open ? 'viewer.sideHide' : 'viewer.sideShow')}
            variant={side.open ? 'default' : 'outline'}
            aria-pressed={side.open}
            data-hint={t(side.open ? 'viewer.sideHide' : 'viewer.sideShow')}
            iconOnly
            onClick={side.onToggle}
          />
          <Separator orientation="vertical" className="mx-1 h-5!" />
        </>
      )}
      {/* Undo and redo first, apart from the photo's own actions (owner, 2026-11-07: they were
          taken for the turn buttons), then copy, paste and reset beside them. */}
      <HistoryButtons canUndo={canUndo} canRedo={canRedo} />
      {/* Copy, paste and reset the photo's settings (FR-097), beside undo (owner, 2026-11-07). */}
      <SettingsButtons photo={photo?.id} ticked={ticked} />
      <Separator orientation="vertical" className="mx-1 h-5!" />
      <ButtonGroup ariaLabel={t('toolbar.rotate')}>
      <ToolbarButton
        icon={<RotateCcwSquare />}
        label={t('viewer.rotateLeftShort')}
        data-hint={t('viewer.rotateLeft', { mod: MOD })}
        iconOnly
        disabled={!photo}
        onClick={() => rotate(false)}
      />
      <ToolbarButton
        icon={<RotateCwSquare />}
        label={t('viewer.rotateRightShort')}
        data-hint={t('viewer.rotateRight', { mod: MOD })}
        iconOnly
        disabled={!photo}
        onClick={() => rotate(true)}
      />
      </ButtonGroup>
      {photo?.queue === 'aside' ? (
        <ToolbarButton
          icon={<ArchiveRestore />}
          label={t('select.toGallery')}
          labelFrom="5xl"
          data-hint={t('preview.toGalleryHint')}
          onClick={() => void commands.setPhotoQueue([photo.id], null)}
        />
      ) : (
        <ToolbarButton
          icon={<ShoppingBasket />}
          label={t(inBasket ? 'preview.inBasket' : 'preview.toBasket')}
          labelFrom="5xl"
          variant={inBasket ? 'default' : 'outline'}
          aria-pressed={inBasket}
          data-hint={t('preview.basketHint')}
          disabled={!photo}
          onClick={() => photo && void commands.setPhotoQueue([photo.id], inBasket ? null : 'print')}
        />
      )}
      <Separator orientation="vertical" className="mx-1 h-5!" />
      {/* Before / After (FR-079, FR-096): the photo without its colour, on or off; its views
          beside it, two buttons apart (owner, 2026-11-08: joined, they looked clumsy). */}
      <ToolbarButton
        icon={<Eye />}
        label={t('look.before')}
        labelFrom="5xl"
        variant={before || compare !== 'off' ? 'default' : 'outline'}
        aria-pressed={before}
        data-hint={t('look.beforeHint')}
        disabled={!photo}
        onClick={onBefore}
      />
      <CompareMenu compare={compare} before={before} onBefore={onBefore} onCompare={onCompare} disabled={!photo} />
      <Separator orientation="vertical" className="mx-1 h-5!" />
      <ToolbarButton
        icon={<Download />}
        label={t('export.title')}
        labelFrom="5xl"
        data-hint={t('viewer.exportHint', { mod: MOD })}
        disabled={!photo}
        onClick={onExport}
      />
      <div className="ml-auto flex items-center gap-2">
        {zoom}
        <ButtonGroup ariaLabel={t('toolbar.panels')}>
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
        </ButtonGroup>
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
    {
      key: 'splitTopBottom',
      icon: <SquareSplitVertical />,
      keys: [],
      on: compare === 'splitTopBottom',
      pick: () => onCompare('splitTopBottom'),
    },
  ] as const
  return (
    <Popover>
      <PopoverTrigger asChild>
        <ToolbarButton icon={<ChevronDown />} label={t('compare.views')} data-hint={t('compare.views')} iconOnly disabled={disabled} />
      </PopoverTrigger>
      <MenuContent align="start">
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
        </MenuContent>
    </Popover>
  )
}
