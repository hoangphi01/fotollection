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

/** `revision` is the settings revision, so the preview refreshes when settings change. */
export function previewUrl(id: number, revision: number): string {
  return `${base}/preview/${id}?rev=${revision}`
}
