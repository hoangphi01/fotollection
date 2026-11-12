// SPDX-License-Identifier: GPL-3.0-only
import { Children, useLayoutEffect, useRef, type ReactNode } from 'react'

/**
 * The type scale's steps (STYLE_RULE §4): a word that does not fit goes one step down, never
 * under `text-2xs` (11 px).
 */
const SCALE = [11, 12, 14, 16, 18, 20, 24]
const SMALLEST = 11

/** The step under a size, or the smallest when there is none. */
function stepDown(px: number): number {
  let next = SMALLEST
  for (const step of SCALE) if (step < px - 0.5) next = step
  return next
}

type Fit = 'full' | 'small' | 'cut'

/** The words a cut control's hint shows: all of them, without a label's closing "…". */
function wholeWords(text: string): string {
  return text.replace(/\s*(…|\.\.\.)$/u, '')
}
const last = new WeakMap<HTMLElement, string>()

/** How wide the words are drawn (a Range over them, never `scrollWidth`: LESSONS). */
function textWidth(span: HTMLElement): number {
  const range = document.createRange()
  range.selectNodeContents(span)
  return range.getBoundingClientRect().width
}

/**
 * Fits a control's words in three steps (FR-125, owner, 2026-11-12): whole; one step of the
 * type scale smaller, for this control only; still too long, cut with an ellipsis and the whole
 * words in its hint. The state is written on the element (`data-fit`), so a check reads it and
 * React never re-renders for it.
 */
function fit(span: HTMLElement, force = false): void {
  const box = span.parentElement
  if (!box) return
  const text = span.textContent ?? ''
  const key = `${box.clientWidth}|${text}`
  if (!force && last.get(span) === key) return
  last.set(span, key)
  span.style.fontSize = ''
  delete span.dataset.hint
  let state: Fit = 'full'
  const room = () => span.getBoundingClientRect().width + 0.5
  if (textWidth(span) > room()) {
    const size = Number.parseFloat(getComputedStyle(span).fontSize)
    const smaller = stepDown(size)
    if (smaller < size) {
      span.style.fontSize = `${smaller}px`
      state = 'small'
    }
    if (textWidth(span) > room()) {
      state = 'cut'
      // A label's own "…" (it opens a dialog) would read as the hint cut too (owner, 2026-11-12).
      span.dataset.hint = wholeWords(text)
    }
  }
  span.dataset.fit = state
}

/** Every control's words on screen, measured again once the fonts arrive. */
const mounted = new Set<HTMLElement>()

// A first measure may run in the fallback face, narrower than Be Vietnam Pro; the control's
// width does not change when the face arrives, so no resize says so (found by MEASURE in hu).
/**
 * Times a pass over some controls' words and writes it on the page (`data-fit-pass`, "count:ms"),
 * so MEASURE reports what fitting costs (OPTIMIZATION §5); nothing else reads it.
 */
function timed(spans: readonly HTMLElement[], force: boolean): void {
  const start = performance.now()
  let n = 0
  for (const span of spans) {
    fit(span, force)
    n++
  }
  document.documentElement.dataset.fitPass = `${n}:${(performance.now() - start).toFixed(2)}`
}

if (typeof document !== 'undefined' && document.fonts) {
  const again = () => timed([...mounted], true)
  document.fonts.addEventListener('loadingdone', again)
  void document.fonts.ready.then(again)
}

// One observer for every control's words, watching the control (its width is what changes),
// never the words themselves, whose size this changes.
const observer =
  typeof ResizeObserver === 'undefined'
    ? null
    : new ResizeObserver((entries) => {
        const spans: HTMLElement[] = []
        for (const entry of entries) {
          const span = (entry.target as HTMLElement).querySelector<HTMLElement>(':scope > [data-fit]')
          if (span) spans.push(span)
        }
        timed(spans, false)
      })

/** A control's words, fitted to its room (FR-125, STYLE_RULE §4 `FitText`). */
export function FitText({ children }: { children: ReactNode }) {
  const ref = useRef<HTMLSpanElement>(null)
  const text = Children.toArray(children).join('')
  useLayoutEffect(() => {
    const span = ref.current
    const box = span?.parentElement
    if (!span || !box) return
    fit(span, true)
    mounted.add(span)
    observer?.observe(box)
    return () => {
      mounted.delete(span)
      observer?.unobserve(box)
    }
  }, [text])
  return (
    <span ref={ref} data-fit="full" className="min-w-0 truncate">
      {children}
    </span>
  )
}
