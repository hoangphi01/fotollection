// SPDX-License-Identifier: GPL-3.0-only

/** True while the user is typing in a field, so single-key shortcuts must stay quiet. */
export function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName))
  )
}

/**
 * True when the focused element uses the arrow keys itself: a text field, or a slider,
 * select or list (Radix renders these as ARIA roles, not form elements). A surrounding
 * shortcut such as ← → to change photo must then leave the key alone.
 */
export function ownsArrowKeys(target: EventTarget | null): boolean {
  return (
    isTyping(target) ||
    (target instanceof Element &&
      target.closest('[role="slider"], [role="combobox"], [role="listbox"], [role="option"]') !== null)
  )
}
