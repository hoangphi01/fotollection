// SPDX-License-Identifier: GPL-3.0-only
// URLs for the Rust `fotothumb` protocol (ARCHITECTURE.md §6). WebView2 on Windows
// addresses custom schemes as http://<scheme>.localhost.

const isWindows = navigator.userAgent.includes('Windows')
const base = isWindows ? 'http://fotothumb.localhost' : 'fotothumb://localhost'

/**
 * `version` changes when a sharper thumbnail becomes available, forcing a reload.
 * `generation` changes when another project replaces the library and ids are reused.
 */
export function thumbUrl(id: number, version = 0, generation = 0): string {
  return `${base}/thumb/${id}?v=${version}&g=${generation}`
}

/** The upright, uncropped photo, for the crop editor. */
export function sourceUrl(id: number, revision: number): string {
  return `${base}/source/${id}?rev=${revision}`
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
): string {
  return `${previewUrl(id, revision)}&region=${rect.x},${rect.y},${rect.w},${rect.h}&px=${px}`
}

/**
 * A collage sheet (FR-035), with the current settings: 640 px for the sheet grid with
 * `small`, 1600 px otherwise.
 */
export function sheetUrl(first: number, second: number | null, revision: number, small = false): string {
  const path = second === null ? `${first}` : `${first}/${second}`
  return `${base}/sheet/${path}?rev=${revision}${small ? '&small=1' : ''}`
}
