// SPDX-License-Identifier: GPL-3.0-only

/** Where the system's own menu and text keys belong: a field, or text marked selectable. */
function editable(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    (target.closest('input, textarea, [contenteditable="true"], .selectable') !== null)
  )
}

/** The web view's keys that make no sense in an app and can lose its state: reload, print,
 * find, view source, the developer tools, caret browsing. The app's own keys pass. */
function browserKey(e: KeyboardEvent): boolean {
  const mod = e.metaKey || e.ctrlKey
  const key = e.key.toLowerCase()
  if (['f3', 'f5', 'f7', 'f12'].includes(key)) return true
  // ⌘R reloads; ⇧⌘R is the app's *Reset settings* (FR-097) and passes.
  if (mod && key === 'r' && !e.shiftKey && !e.altKey) return true
  if (mod && ['p', 'u', 'f', 'g'].includes(key) && !e.shiftKey && !e.altKey) return true
  // The developer tools' keys (off in releases anyway); Ctrl+Shift+C is the app's *Copy
  // settings* on Windows (FR-097) and passes.
  if (e.ctrlKey && e.shiftKey && ['i', 'j'].includes(key) && !e.metaKey) return true
  if (e.metaKey && e.altKey && ['i', 'j', 'c', 'u'].includes(key)) return true
  return false
}

/**
 * Makes the web view behave as an app (owner, 2026-11-08, SECURITY.md): no web context menu
 * (Look Up, Translate, Inspect…) except over a field or selectable text, where Copy and Paste
 * stay; no browser keys; no image or link dragged out. The app's own menus call
 * `preventDefault` themselves and open their own. Called once, before the app renders.
 */
export function noWeb(): void {
  document.addEventListener(
    'contextmenu',
    (e) => {
      if (!editable(e.target)) e.preventDefault()
    },
    true,
  )
  window.addEventListener(
    'keydown',
    (e) => {
      if (browserKey(e) && !editable(e.target)) e.preventDefault()
    },
    true,
  )
  document.addEventListener(
    'dragstart',
    (e) => {
      const t = e.target
      // The app's own drags (folders, collage cells) set `draggable` themselves.
      if (t instanceof HTMLImageElement || t instanceof HTMLAnchorElement) e.preventDefault()
    },
    true,
  )
}
