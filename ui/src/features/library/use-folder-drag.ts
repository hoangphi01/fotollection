// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent, type PointerEvent } from 'react'

/** Movement before a press becomes a drag, in px, as a click must stay a click. */
const SLOP = 4
/** A press held this long becomes a drag without moving (owner: "hold, or press and drag"). */
const HOLD_MS = 200

type Drag = {
  id: number
  pointer: number
  startY: number
  dy: number
  active: boolean
  /** The rows' top and height when the press began, in the list's order. */
  rows: { top: number; height: number }[]
  from: number
}

/**
 * Folder rows reordered by dragging, as Chrome's tabs (FR-107): a press that moves 4 px or is
 * held 200 ms lifts the row; it follows the pointer and the rows it passes slide aside; on
 * release `onMove(id, to)` asks Rust, and the dropped order shows until Rust's list arrives,
 * so nothing jumps back. Escape puts it back; Alt+↑/↓ moves the focused row one place. Pointer
 * events, not HTML drag and drop, which the webview gives to dropped folders.
 */
export function useFolderDrag(ids: readonly number[], onMove: (id: number, to: number) => void) {
  const [drag, setDrag] = useState<Drag | null>(null)
  // The order dropped, for the list it was dropped on: shown until Rust's answer differs.
  const [dropped, setDropped] = useState<{ basedOn: string; order: number[] } | null>(null)
  const key = ids.join(',')
  const order = dropped && dropped.basedOn === key ? dropped.order : [...ids]
  const rowRefs = useRef(new Map<number, HTMLElement>())
  const hold = useRef<number | null>(null)
  // A drag ends with a click on the row; that click must not open the folder.
  const suppressClick = useRef(false)

  const target = (d: Drag) => {
    const self = d.rows[d.from]
    if (!self) return d.from
    const centre = self.top + self.height / 2 + d.dy
    let to = 0
    d.rows.forEach((r, i) => {
      if (i !== d.from && r.top + r.height / 2 < centre) to += 1
    })
    return to
  }

  useEffect(() => {
    if (!drag?.active) return
    const onKey = (e: globalThis.KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        suppressClick.current = true
        setDrag(null)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [drag?.active])

  const commit = (id: number, to: number, from: number) => {
    if (to === from) return
    const next = order.filter((f) => f !== id)
    next.splice(to, 0, id)
    setDropped({ basedOn: key, order: next })
    onMove(id, to)
  }

  const rowProps = (id: number) => ({
    ref: (el: HTMLElement | null) => {
      if (el) rowRefs.current.set(id, el)
      else rowRefs.current.delete(id)
    },
    style: style(id),
  })

  const style = (id: number): CSSProperties | undefined => {
    if (!drag?.active) return undefined
    const i = order.indexOf(id)
    if (id === drag.id) return { transform: `translateY(${drag.dy}px)`, zIndex: 10, position: 'relative' }
    const to = target(drag)
    const step = (drag.rows[drag.from]?.height ?? 0) + 4 // a row and the list's 4 px gap
    if (drag.from < to && i > drag.from && i <= to) return { transform: `translateY(${-step}px)` }
    if (drag.from > to && i >= to && i < drag.from) return { transform: `translateY(${step}px)` }
    return undefined
  }

  const handleProps = (id: number) => ({
    onPointerDown: (e: PointerEvent<HTMLElement>) => {
      if (e.button !== 0) return
      const rows = order.map((f) => {
        const r = rowRefs.current.get(f)?.getBoundingClientRect()
        return { top: r?.top ?? 0, height: r?.height ?? 0 }
      })
      setDrag({ id, pointer: e.pointerId, startY: e.clientY, dy: 0, active: false, rows, from: order.indexOf(id) })
      e.currentTarget.setPointerCapture(e.pointerId)
      if (hold.current !== null) window.clearTimeout(hold.current)
      hold.current = window.setTimeout(() => setDrag((d) => (d && d.id === id ? { ...d, active: true } : d)), HOLD_MS)
    },
    onPointerMove: (e: PointerEvent<HTMLElement>) => {
      setDrag((d) => {
        if (!d || d.pointer !== e.pointerId) return d
        const dy = e.clientY - d.startY
        return { ...d, dy, active: d.active || Math.abs(dy) > SLOP }
      })
    },
    onPointerUp: (e: PointerEvent<HTMLElement>) => {
      if (hold.current !== null) window.clearTimeout(hold.current)
      if (drag && drag.pointer === e.pointerId && drag.active) {
        suppressClick.current = true
        commit(drag.id, target(drag), drag.from)
      }
      setDrag(null)
    },
    onPointerCancel: () => {
      if (hold.current !== null) window.clearTimeout(hold.current)
      setDrag(null)
    },
    onKeyDown: (e: KeyboardEvent<HTMLElement>) => {
      if (!e.altKey || (e.key !== 'ArrowUp' && e.key !== 'ArrowDown')) return
      e.preventDefault()
      const from = order.indexOf(id)
      const to = Math.max(0, Math.min(order.length - 1, from + (e.key === 'ArrowUp' ? -1 : 1)))
      commit(id, to, from)
    },
  })

  /** True once for the click that ends a drag; the row's click handler skips it. */
  const takeSuppressedClick = () => {
    const was = suppressClick.current
    suppressClick.current = false
    return was
  }

  return { order, dragging: drag?.active ? drag.id : null, rowProps, handleProps, takeSuppressedClick }
}
