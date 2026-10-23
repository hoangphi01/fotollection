// SPDX-License-Identifier: GPL-3.0-only
// URLs for the Rust `fotothumb` protocol (ARCHITECTURE.md §6). WebView2 on Windows
// addresses custom schemes as http://<scheme>.localhost.

const isWindows = navigator.userAgent.includes('Windows')
const base = isWindows ? 'http://fotothumb.localhost' : 'fotothumb://localhost'

/** `version` changes when a sharper thumbnail becomes available, forcing a reload. */
export function thumbUrl(id: number, version = 0): string {
  return `${base}/thumb/${id}?v=${version}`
}

/** `revision` is the settings revision, so the preview refreshes when settings change. */
export function previewUrl(id: number, revision: number): string {
  return `${base}/preview/${id}?rev=${revision}`
}
