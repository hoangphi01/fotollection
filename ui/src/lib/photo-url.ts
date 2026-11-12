// SPDX-License-Identifier: GPL-3.0-only
// URLs for the Rust `fotothumb` protocol (ARCHITECTURE.md §6). WebView2 on Windows
// addresses custom schemes as http://<scheme>.localhost.

const isWindows = navigator.userAgent.includes('Windows')
const base = isWindows ? 'http://fotothumb.localhost' : 'fotothumb://localhost'

/**
 * `version` changes when a sharper thumbnail becomes available, forcing a reload.
 * `generation` changes when another project replaces the library and ids are reused.
 * `look` is the photo's colour key (`PhotoSummary.look`, FR-114): Rust serves the thumbnail
 * through the photo's colour, and a new colour is a new address, so it is fetched again.
 */
export function thumbUrl(id: number, version = 0, generation = 0, look = 0): string {
  return `${base}/thumb/${id}?v=${version}&g=${generation}&l=${look}`
}

/** The upright, uncropped photo, for the crop editor; `sheet` turns it as it sits in its collage cell. */
export function sourceUrl(id: number, revision: number, sheet = false): string {
  return `${base}/source/${id}?rev=${revision}${sheet ? '&sheet=1' : ''}`
}

/** The connected camera's latest live-view frame (FR-062); `seq` comes with `camera-frame`. */
export function liveViewUrl(seq: number): string {
  return `${base}/camera/live?seq=${seq}`
}

/** `revision` is the settings revision, so the preview refreshes when settings change. */
export function previewUrl(id: number, revision: number): string {
  return `${base}/preview/${id}?rev=${revision}`
}

/**
 * One part of the full-resolution preview, for zoom (FR-057): `rect` in fractions of the
 * photo, `px` the widest it should come back.
 */
export function regionUrl(
  id: number,
  revision: number,
  rect: { x: number; y: number; w: number; h: number },
  px: number,
  /** Without the colour look: a compare view's *Before* side (FR-096). */
  plain = false,
): string {
  return `${previewUrl(id, revision)}${plain ? '&plain=1' : ''}&region=${rect.x},${rect.y},${rect.w},${rect.h}&px=${px}`
}

/**
 * A collage sheet (FR-035), with the current settings: 640 px for the sheet grid with
 * `small`, 1600 px otherwise; with `clip`, its photos' clipped pixels marked (FR-098, `J`).
 */
/** Which photos of a sheet are drawn without their look (FR-096, *Hold for the original*). */
export type SheetPlain = 'both' | 'first' | 'second'

export function sheetUrl(first: number, second: number | null, revision: number, small = false, clip = false, plain: SheetPlain | null = null): string {
  const path = second === null ? `${first}` : `${first}/${second}`
  return `${base}/sheet/${path}?rev=${revision}${small ? '&small=1' : ''}${plain ? `&plain=${plain}` : clip ? '&clip=1' : ''}`
}

/** Where the preview shows the photo itself, a grey PNG, for the look on the GPU (ADR-0039). */
export function maskUrl(id: number, revision: number): string {
  return `${previewUrl(id, revision)}&mask=1`
}

/** The look's grain and vignette, an RGB PNG of the preview, for the look on the GPU (FR-079). */
export function effectsUrl(id: number, revision: number): string {
  return `${previewUrl(id, revision)}&effects=1`
}

/** The photo's colour look as a 33³ RGBA8 table; empty when it changes nothing (ADR-0039). */
export function lookTableUrl(id: number, revision: number): string {
  return `${base}/look/${id}?rev=${revision}`
}

/**
 * The photo through one look of the library, small and square, for the Looks card (FR-079).
 * Kept by the webview: a tile changes only with the photo or the look, never with a slider.
 */
export function lookTileUrl(id: number, look: string): string {
  // Library ids are lowercase letters, digits, dashes and one colon: safe in a query as they are.
  return `${base}/looktile/${id}?look=${look}`
}

/** One look of the library's table for this photo, for the hover preview (FR-089). */
export function lookPreviewTableUrl(id: number, look: string): string {
  return `${base}/looktable/${id}?look=${look}`
}
