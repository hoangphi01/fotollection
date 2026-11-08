// SPDX-License-Identifier: GPL-3.0-only
import { Crop, Download, Images, Printer, RotateCw, RotateCwSquare, ShoppingBasket, SlidersHorizontal } from 'lucide-react'
import { Spinner } from '@/components/forms/spinner'
import { useEffect, useRef, useState, type KeyboardEvent, type MouseEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { SectionCard } from '@/components/layout/section-card'
import { InspectorSection } from '@/components/layout/inspector-section'
import { InspectorTabs } from '@/components/layout/inspector-tabs'
import { KeyHints } from '@/components/layout/key-hints'
import { KeysInfo } from '@/components/layout/keys-info'
import { StageNav } from '@/components/layout/stage-nav'
import { ButtonGroup } from '@/components/forms/button-group'
import { ToolbarButton } from '@/components/forms/toolbar-button'
import { Kbd } from '@/components/ui/kbd'
import { Separator } from '@/components/ui/separator'
import { HistoryButtons } from '@/features/history/history-buttons'
import { StageSpinner } from '@/components/layout/stage-spinner'
import { ViewerFrame } from '@/components/layout/viewer-frame'
import { commands, type CullFilter, type PhotoId, type PhotoSummary, type Queue, type RenderSettings } from '@/ipc/bindings'
import { ExportSection } from '@/features/export/export-section'
import type { ExportApi } from '@/features/export/use-export'
import { LookStrip } from '@/features/look/look-strip'
import { PhotoLookCards } from '@/features/look/photo-look-cards'
import { useLookPicker } from '@/features/look/use-looks'
import { SettingsButtons } from '@/features/develop/settings-buttons'
import { HistogramCard } from '@/features/viewer/histogram-card'
import { setSettingsTarget } from '@/features/develop/settings-copy'
import { DateStampCard } from '@/features/stamp/date-stamp-card'
import { CropEditor } from '@/features/viewer/crop-editor'
import { sendTo } from '@/features/viewer/send-to'
import { useShownSrc } from '@/features/viewer/use-shown-src'
import { useZoom } from '@/features/viewer/use-zoom'
import { ZoomControls } from '@/features/viewer/zoom-controls'
import { formatCapture, formatNumber } from '@/lib/format'
import { ownsArrowKeys } from '@/lib/keyboard'
import { sheetUrl, sourceUrl, thumbUrl } from '@/lib/photo-url'
import { MOD } from '@/lib/platform'
import { cn } from '@/lib/utils'
import { CHOSEN_ROW, ListRow } from '@/components/forms/list-row'
import { CollageCellsCard } from './collage-cells-card'
import { PostcardCard } from './postcard-card'
import { PostcardCutBadge } from './postcard-cut-badge'
import { PostcardTextCard } from './postcard-text-card'
import { sheetCells } from './sheet-cells'
import { SheetCellsOverlay } from './sheet-cells-overlay'
import { SheetSizeCard } from './sheet-size-card'
import { useCellGeometry } from './use-cell-geometry'
import type { CollageApi } from './use-collage'
import { usePostcardCuts } from './use-postcard-cuts'
import { useSheetInfo } from './use-sheet-info'
import { useSheetShape } from './use-sheet-shape'

type SheetPreviewDialogProps = {
  collage: CollageApi
  /** The preview is open on the chosen sheet. */
  open: boolean
  photos: PhotoSummary[]
  settings: RenderSettings | null
  onSettingsChange: (change: (current: RenderSettings) => RenderSettings) => void
  /** After a photo's look became everyone's (FR-079). */
  onSettingsReload: () => void
  exporter: ExportApi
  filter: CullFilter
  /** Photos with their own stamp from the preview (FR-044). */
  customStamps: number
  revision: number
  generation: number
  /** Photos ticked in Collage, for *Paste settings* (FR-097). */
  ticked: readonly PhotoId[]
  /** Undo and redo in its toolbar, as the photo preview's. */
  canUndo: boolean
  canRedo: boolean
  onClose: () => void
}

/**
 * A collage sheet as it will print (FR-035), in the same frame as the photo preview: zoom,
 * the same cards as the Collage panel in the same order, the export of this sheet only, then
 * the sheet's two photos with their basket and crop. A photo's crop here is its crop on
 * sheets, apart from its print crop.
 */
/** The inspector's tabs, as the photo preview's (FR-095, owner 2026-11-07): the colour of
 * the sheet or of a photo on it, then its print. Kept while the app runs. */
type SheetTab = 'edit' | 'print'
let lastTab: SheetTab = 'edit'

export function SheetPreviewDialog({
  collage,
  open,
  photos,
  settings,
  onSettingsChange,
  onSettingsReload,
  exporter,
  filter,
  customStamps,
  revision,
  generation,
  ticked,
  canUndo,
  canRedo,
  onClose,
}: SheetPreviewDialogProps) {
  const { t } = useTranslation()
  const { sheets, index, sheet } = collage
  const key = sheet ? `${sheet.first}:${sheet.second}` : undefined
  // J: the sheet's photos with their clipped pixels marked (FR-098), never the card or paper.
  const [clipping, setClipping] = useState(false)
  // W: the white-balance picker armed for the picked photo; the next click on it sets its
  // warmth and tint, a click elsewhere on the sheet is refused (ERR-IPC-0026).
  const [picking, setPicking] = useState(false)
  const [tab, setTabState] = useState<SheetTab>(lastTab)
  const setTab = (next: SheetTab) => {
    lastTab = next
    setTabState(next)
  }
  const src = sheet ? sheetUrl(sheet.first, sheet.second, revision, false, clipping) : ''
  const { src: shownSrc, error: shownError } = useShownSrc(key, src)
  const shape = useSheetShape(shownSrc ?? '')
  const [stage, setStage] = useState<HTMLDivElement | null>(null)
  const [cropping, setCropping] = useState(false)
  const zoom = useZoom(stage, `${key}:${cropping}`)
  const contentRef = useRef<HTMLDivElement>(null)
  // No photo is chosen until the user picks one, so the sheet shows as it prints.
  const [picked, setPicked] = useState<{ key: string | undefined; cell: 0 | 1 } | null>(null)
  const chosen = picked && picked.key === key ? picked.cell : null
  const choose = (cell: 0 | 1 | null) => {
    if (cell !== chosen) setPicking(false)
    setPicked(cell === null ? null : { key, cell })
  }
  const photo = sheet && chosen !== null ? (chosen === 0 ? sheet.first : sheet.second) : null
  const pickAt = (event: MouseEvent<HTMLImageElement>) => {
    const box = event.currentTarget.getBoundingClientRect()
    if (!sheet || photo === null || box.width === 0 || box.height === 0) return
    const fx = (event.clientX - box.left) / box.width
    const fy = (event.clientY - box.top) / box.height
    setPicking(false)
    void commands.pickWhiteBalanceOnSheet({ first: sheet.first, second: sheet.second }, photo, fx, fy)
  }
  // The picked photo is the one copy, paste and reset act on, its crop the collage crop (FR-097).
  useEffect(() => {
    if (!open || photo === null) return
    const target = { photo, collage: true }
    setSettingsTarget(target)
    return () => setSettingsTarget(null, target)
  }, [open, photo])
  // The strip of looks under the sheet (owner, 2026-11-07): the picked photo's, or the whole
  // sheet's when none is picked; the first photo's look marks the chosen tile.
  const sheetIds = sheet ? (sheet.second === null ? [sheet.first] : [sheet.first, sheet.second]) : undefined
  const looks = useLookPicker(photo ?? sheet?.first, revision, photo === null ? sheetIds : undefined)
  const info = useSheetInfo(open ? sheet : undefined, revision)
  const cell = info?.cells.find((c) => c.id === photo) ?? info?.cells[0]
  const geometry = useCellGeometry(cropping ? photo : null, sheet, revision)
  const summary = (id: PhotoId | null) => (id === null ? undefined : photos.find((p) => p.id === id))
  const cells = sheet ? sheetCells(sheet) : []

  useEffect(() => {
    if (!cropping) contentRef.current?.focus()
  }, [cropping])

  // ↑ to the print basket, ↓ set aside, for the chosen photo (FR-050); the choice moves on
  // to the next photo of the sheets, as in the photo preview.
  const nextPhoto = () => {
    if (chosen === 0 && sheet?.second != null) choose(1)
    else go(1)
  }
  const send = (queue: Queue) => {
    if (photo !== null) sendTo(photo, queue, filter.queue ?? 'gallery', nextPhoto)
  }
  const go = (delta: number) => {
    setCropping(false)
    setPicking(false)
    choose(null)
    collage.select(Math.min(sheets.length - 1, Math.max(0, index + delta)), 0)
  }
  // A quarter turn of the photo in its cell only (FR-035): its print keeps its orientation.
  const postcard = settings?.collage?.postcard?.enabled ?? false
  const cut = usePostcardCuts(settings, revision).has(sheet?.first ?? -1)
  const rotatePhoto = (id: PhotoId) => void commands.rotateCollagePhoto(id)
  const onKeyDown = (event: KeyboardEvent) => {
    if (cropping || ownsArrowKeys(event.target)) return
    if (zoom.onKey(event)) return
    const pressed = event.key.toLowerCase()
    if (event.key === 'ArrowRight') go(1)
    else if (event.key === 'ArrowLeft') go(-1)
    else if (event.key === 'ArrowUp') send('print')
    else if (event.key === 'ArrowDown') send('aside')
    else if (pressed === '1' || pressed === '2') choose(pressed === '1' ? 0 : 1)
    // 0 lets go of the photo: the colour cards then edit the whole sheet (owner, 2026-11-07).
    else if (pressed === '0') choose(null)
    else if (pressed === 'e' && !event.metaKey && !event.ctrlKey) setTab('edit')
    else if (pressed === 'c' && photo !== null) setCropping(true)
    // J and W by the key's place, as in the photo preview (FR-098); W needs a picked photo.
    else if (event.code === 'KeyJ' && !event.metaKey && !event.ctrlKey && !event.altKey) setClipping((on) => !on)
    else if (event.code === 'KeyW' && photo !== null && !event.metaKey && !event.ctrlKey && !event.altKey) setPicking((on) => !on)
    // ⇧R turns the chosen photo in its cell; R alone turns the sheet.
    else if (pressed === 'r' && event.shiftKey && photo !== null) rotatePhoto(photo)
    else if (pressed === 'r' && sheet && !postcard && !event.metaKey && !event.ctrlKey) collage.turn(sheet.first)
    else return
    event.preventDefault()
  }
  const names = cells.flatMap(([id]) => summary(id)?.fileName ?? [])

  return (
    <ViewerFrame
      open={open && sheet !== undefined}
      onClose={onClose}
      contentRef={contentRef}
      onKeyDown={onKeyDown}
      onEscapeKeyDown={(e) => {
        // Esc leaves the picker, then the crop editor, then lets go of the chosen photo, then closes.
        if (picking) {
          e.preventDefault()
          setPicking(false)
        } else if (cropping) {
          e.preventDefault()
          setCropping(false)
        } else if (chosen !== null) {
          e.preventDefault()
          choose(null)
        }
      }}
      stageRef={setStage}
      below={sheet && !cropping ? <LookStrip photoId={photo ?? sheet.first} picker={looks} numbered={false} folding /> : undefined}
      tabs={
        <InspectorTabs<SheetTab>
          ariaLabel={t('inspector.tabs')}
          value={tab}
          onChange={setTab}
          groups={[
            [{ value: 'edit', icon: <SlidersHorizontal aria-hidden />, label: t('inspector.edit') }],
            [{ value: 'print', icon: <Printer aria-hidden />, label: t('inspector.print') }],
          ]}
        />
      }
      // The sheet's tools in a row above the stage, never over the sheet, in the photo
      // preview's order (owner, 2026-11-07; STYLE_RULE §4): undo and redo, the chosen photo's
      // settings, then crop and turns, then zoom.
      toolbar={
        <div className="@container flex h-header items-center gap-2 overflow-x-auto overflow-y-hidden px-3 [scrollbar-width:thin] [&>*]:shrink-0">
          <HistoryButtons canUndo={canUndo} canRedo={canRedo} />
          <SettingsButtons photo={photo ?? undefined} ticked={ticked} collage />
          <Separator orientation="vertical" className="mx-1 h-5!" />
          {/* The sheet's own tools joined (owner, 2026-11-08). */}
          <ButtonGroup ariaLabel={t('toolbar.sheetTools')}>
          <ToolbarButton
            icon={<Crop />}
            label={t('viewer.crop')}
            data-hint={photo === null ? t('collage.pickToCrop') : t('viewer.crop')}
            disabled={photo === null || cropping}
            onClick={() => setCropping(true)}
          />
          {/* A postcard is held the way the panel says: only the photo turns. */}
          {!postcard && (
            <ToolbarButton
              icon={<RotateCw />}
              label={t('collage.turn')}
              data-hint={t('collage.turnHint')}
              disabled={!sheet || cropping}
              onClick={() => sheet && collage.turn(sheet.first)}
            />
          )}
          {/* Turns the chosen photo in its cell, so a landscape photo fills a standing cell. */}
          <ToolbarButton
            icon={<RotateCwSquare />}
            label={t('collage.rotatePhoto')}
            data-hint={photo === null ? t('collage.pickToRotate') : t('collage.rotatePhotoHint')}
            disabled={photo === null || cropping}
            onClick={() => photo !== null && rotatePhoto(photo)}
          />
          </ButtonGroup>
          <div className="ml-auto flex items-center gap-2">
            <ZoomControls
              inline
              percent={zoom.percent}
              zoomed={zoom.zoomed}
              canZoomIn={zoom.canZoomIn && !cropping}
              loading={false}
              mod={MOD}
              onZoomIn={zoom.zoomIn}
              onZoomOut={zoom.zoomOut}
              onFit={zoom.reset}
            />
            {/* Every key behind ⓘ, as the photo preview's (owner, 2026-11-07: a paragraph of
                keys is hard to read); the footer keeps the few used most. */}
            <KeysInfo lines={t('collage.sheetKeys', { mod: MOD }).split('\n')} />
          </div>
        </div>
      }
      // A click on the ground around the sheet lets go of the chosen photo, like Esc.
      onStageGroundClick={() => !cropping && !picking && choose(null)}
      title={t('collage.sheetOf', { index: formatNumber(index + 1), count: formatNumber(sheets.length) })}
      description={names.length === 2 ? t('collage.pairNames', { first: names[0], second: names[1] }) : names[0]}
      footer={
        <KeyHints
          hints={
            cropping
              ? [
                  { keys: ['←', '→'], label: t('viewer.level') },
                  { keys: ['Enter'], label: t('common.done') },
                  { keys: ['Esc'], label: t('viewer.leaveCrop') },
                ]
              : [
                  { keys: ['←', '→'], label: t('collage.browse') },
                  { keys: ['1', '2'], label: t('collage.choose') },
                  { keys: ['0'], label: t('collage.wholeSheet') },
                  { keys: ['Esc'], label: t('viewer.close') },
                  { keys: ['ⓘ'], label: t('viewer.everyKey') },
                ]
          }
        />
      }
      stage={
        cropping && geometry && photo !== null ? (
          <CropEditor
            key={photo}
            geometry={geometry}
            sourceSrc={sourceUrl(photo, revision, true)}
            onCancel={() => setCropping(false)}
            onCommit={(crop, straighten) => {
              setCropping(false)
              // The crop on sheets, apart from the print crop (FR-035).
              void commands.setCollageCrop(photo, crop, straighten)
            }}
          />
        ) : (
          <>
            {shownSrc && sheet ? (
              <div className="relative max-h-full max-w-full" style={{ transform: zoom.transform }}>
                <img
                  src={shownSrc}
                  alt={t('collage.sheetLabel')}
                  draggable={false}
                  onLoad={shape.onLoad}
                  // While W is armed, a click picks: no zoom drag, no cell overlay (LESSONS: two
                  // gestures on one surface turn each other off).
                  onPointerDown={picking ? undefined : zoom.pointer.onPointerDown}
                  onPointerMove={picking ? undefined : zoom.pointer.onPointerMove}
                  onPointerUp={picking ? undefined : zoom.pointer.onPointerUp}
                  onClick={picking ? pickAt : undefined}
                  className={cn(
                    // The dialog's 90vh less the stage's padding (2 × 2rem), the toolbar row above
                    // it (`--spacing-header`) and its line, and the strip of looks below (7.75rem,
                    // `STRIP_OPEN`, and its line): the whole sheet fits, any shape.
                    'block max-h-[calc(90vh-4rem-var(--spacing-header)-1px-7.75rem-1px)] max-w-full animate-in object-contain stage-photo fade-in-0 motion-in',
                    picking && 'cursor-crosshair',
                  )}
                />
                {!zoom.zoomed && !picking && (
                  <SheetCellsOverlay sheet={sheet} sideBySide={shape.sideBySide} postcard={postcard}>
                    {(id, c) => (
                      <button
                        key={c}
                        type="button"
                        disabled={id === null}
                        aria-label={summary(id)?.fileName ?? t('collage.emptyCell')}
                        className={cn('flex-1 rounded-sm', chosen === c && id !== null && 'ring-2 ring-primary ring-inset')}
                        onClick={() => choose(chosen === c ? null : c)}
                        onDoubleClick={() => {
                          if (id === null) return
                          choose(c)
                          setCropping(true)
                        }}
                      />
                    )}
                  </SheetCellsOverlay>
                )}
                {cut && <PostcardCutBadge className="absolute top-2 left-2" />}
              </div>
            ) : (
              <StageSpinner error={shownError} />
            )}
            <StageNav to="previous" label={t('collage.previous')} disabled={index === 0} onClick={() => go(-1)} />
            <StageNav to="next" label={t('collage.next')} disabled={index >= sheets.length - 1} onClick={() => go(1)} />
            <span className="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-stage-chip px-3 py-1 text-xs text-stage-ink tabular-nums">
              {/* The new render after a change of setting is on its way; the spinner keeps
                  its place when hidden, so the text never shifts. */}
              <Spinner
                className={cn('size-3', (shownSrc === null || shownSrc === src) && 'invisible')}
                aria-label={shownSrc !== null && shownSrc !== src ? t('viewer.rendering') : undefined}
                aria-hidden={shownSrc === null || shownSrc === src}
              />
              {t('collage.position', { index: formatNumber(index + 1), count: formatNumber(sheets.length) })}
            </span>
          </>
        )
      }
    >
      {/* Two tabs as the photo preview's (owner, 2026-11-07): Edit, the sheet's photos to pick
          from (none picked: the whole sheet), then the colour, in the photo preview's order;
          Print, the cards the Collage panel shows, in its order (STYLE_RULE §4). */}
      {tab === 'edit' && (
        <>
      <InspectorSection icon={<Images className="size-4" />} title={t('collage.photosOnSheet')}>
        <ListRow icon={<Images />} chosen={chosen === null} onClick={() => choose(null)} trailing={<Kbd>0</Kbd>}>
          {t('collage.wholeSheet')}
        </ListRow>
        {cells.map(([id, c]) => {
          const p = summary(id)
          return (
            <div key={c} className={cn('flex items-center gap-2 rounded-md p-1', chosen === c && p && CHOSEN_ROW)}>
              {p ? (
                <>
                  <button type="button" className="shrink-0" onClick={() => choose(chosen === c ? null : c)} aria-label={p.fileName}>
                    <img src={thumbUrl(p.id, 0, generation, p.look)} alt="" className="size-10 rounded object-cover" />
                  </button>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm" data-hint={p.fileName}>{p.fileName}</p>
                    <p className="text-xs text-muted-foreground">{formatCapture(p.capture)}</p>
                  </div>
                  <Button
                    variant={p.queue === 'print' ? 'default' : 'outline'}
                    size="icon-sm"
                    aria-pressed={p.queue === 'print'}
                    aria-label={t(p.queue === 'print' ? 'preview.inBasket' : 'preview.toBasket')}
                    data-hint={t(p.queue === 'print' ? 'preview.inBasket' : 'preview.toBasket')}
                    onClick={() => void commands.setPhotoQueue([p.id], p.queue === 'print' ? null : 'print')}
                  >
                    <ShoppingBasket />
                  </Button>
                  <Button
                    variant="outline"
                    size="icon-sm"
                    aria-label={t('viewer.crop')}
                    data-hint={t('viewer.crop')}
                    onClick={() => {
                      choose(c)
                      setCropping(true)
                    }}
                  >
                    <Crop />
                  </Button>
                </>
              ) : (
                <p className="text-xs text-muted-foreground">{t('collage.emptyCell')}</p>
              )}
            </div>
          )
        })}
      </InspectorSection>
      {photo !== null ? (
        <>
          {/* The picked photo's histogram (FR-098), then its settings (FR-097); W armed, what
              the next click does instead. The curves are the photo's own preview's. */}
          <HistogramCard
            photoId={photo}
            revision={revision}
            clipping={clipping}
            onClipping={setClipping}
            picking={picking}
            onPicking={setPicking}
          />
          <PhotoLookCards photoId={photo} revision={revision} onAppliedToAll={onSettingsReload} />
        </>
      ) : (
        sheet && (
          <PhotoLookCards
            photoId={sheet.first}
            sheet={sheetIds}
           
            revision={revision}
            onAppliedToAll={onSettingsReload}
          />
        )
      )}
        </>
      )}
      {tab === 'print' && (
        <>
      <SheetSizeCard settings={settings} onChange={onSettingsChange} info={info ?? undefined} />
      <PostcardCard settings={settings} onChange={onSettingsChange} />
      <PostcardTextCard
        settings={settings}
        onChange={onSettingsChange}
        photo={sheet?.first}
        revision={revision}
        filter={filter}
        ticked={ticked}
      />
      <CollageCellsCard settings={settings} onChange={onSettingsChange} cell={cell} />
      <DateStampCard
        settings={settings}
        onChange={onSettingsChange}
        customStamps={customStamps}
        photos={cells.flatMap(([id]) => (id === null ? [] : [{ id, name: summary(id)?.fileName ?? '' }]))}
        revision={revision}
      />
      {sheet && (
        <SectionCard icon={<Download />} title={t('export.title')}>
          <ExportSection
            exporter={exporter}
            photoCount={sheet.second === null ? 1 : 2}
            filter={filter}
            only={sheet.first}
            revision={revision}
            sheets={1}
          />
        </SectionCard>
      )}
        </>
      )}
    </ViewerFrame>
  )
}
