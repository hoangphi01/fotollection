// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, type RefObject } from 'react'

/**
 * Keeps what the user is looking at in place when the content above it changes height (the
 * browser's scroll anchoring, which WebKit lacks; owner, 2026-11-19: RollSense's calendar
 * drifted while the cards above it filled). The anchor is the element last pressed in the
 * content, else the one at the middle of the view after the user's last scroll; whenever the content
 * resizes, the view scrolls by as much as the anchor moved. Nothing moves before the user
 * touched the area, so a panel that opens at its top stays there.
 */
export function useScrollAnchor(content: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const box = content.current
    const view = box?.closest<HTMLElement>('[data-slot="scroll-area-viewport"]')
    if (!box || !view) return
    let anchor: Element | null = null
    let top = 0
    // The scroll this hook makes is not the user's: it keeps the anchor.
    let own = false
    const keep = (el: Element | null) => {
      anchor = el && box.contains(el) ? el : null
      if (anchor) top = anchor.getBoundingClientRect().top
    }
    const onScroll = () => {
      if (own) {
        own = false
        return keep(anchor)
      }
      // The user scrolled: what is now at the middle of the view.
      const r = view.getBoundingClientRect()
      keep(document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2))
    }
    const onPress = (e: PointerEvent) => keep(e.target as Element)
    const observer = new ResizeObserver(() => {
      if (!anchor?.isConnected) return
      const moved = anchor.getBoundingClientRect().top - top
      if (Math.abs(moved) <= 0.5) return
      own = true
      view.scrollTop += moved
      top = anchor.getBoundingClientRect().top
    })
    observer.observe(box)
    view.addEventListener('scroll', onScroll, { passive: true })
    box.addEventListener('pointerdown', onPress)
    return () => {
      observer.disconnect()
      view.removeEventListener('scroll', onScroll)
      box.removeEventListener('pointerdown', onPress)
    }
  }, [content])
}
