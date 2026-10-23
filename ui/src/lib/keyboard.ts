// SPDX-License-Identifier: GPL-3.0-only

/** True while the user is typing in a field, so single-key shortcuts must stay quiet. */
export function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName))
  )
}
