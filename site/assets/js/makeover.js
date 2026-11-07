// SPDX-License-Identifier: GPL-3.0-only
// The landing page's makeover (owner, 2026-11-06): v0.12's gallery is wiped into v0.13's once
// it comes into view, then the line follows a mouse passing over the picture, or a finger dragged across it
// (pointer events; a vertical swipe still scrolls the page, touch-action: pan-y), and the arrow
// keys move it through a range input kept for the keyboard. With reduced motion the new one
// stands at once; without script nothing moves.
for (const figure of document.querySelectorAll('[data-makeover]')) {
  const stage = figure.querySelector('.makeover-stage')
  const range = figure.querySelector('.makeover-range')
  const set = (pos) => {
    const clamped = Math.max(0, Math.min(100, pos))
    figure.style.setProperty('--pos', `${clamped}%`)
    range.value = String(Math.round(clamped))
  }
  const fromPointer = (event) => {
    const box = stage.getBoundingClientRect()
    set(((event.clientX - box.left) / box.width) * 100)
  }
  const ready = () => {
    range.hidden = false
    figure.style.setProperty('--handle', '1')
    range.addEventListener('input', () => set(Number(range.value)))
    let dragging = null
    // The browser's own image drag would take the press away (owner, 2026-11-07): never.
    stage.addEventListener('dragstart', (event) => event.preventDefault())
    stage.addEventListener('pointerdown', (event) => {
      if (event.pointerType === 'mouse') event.preventDefault()
      dragging = event.pointerId
      stage.setPointerCapture(event.pointerId)
      fromPointer(event)
    })
    stage.addEventListener('pointermove', (event) => {
      // A mouse moves the line as it passes, no press needed; a finger drags it.
      if (event.pointerId === dragging || event.pointerType === 'mouse') fromPointer(event)
    })
    const stop = (event) => {
      if (event.pointerId === dragging) dragging = null
    }
    stage.addEventListener('pointerup', stop)
    stage.addEventListener('pointercancel', stop)
  }
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
    ready()
    continue
  }
  set(0)
  const seen = new IntersectionObserver(
    ([entry]) => {
      if (!entry.isIntersecting) return
      seen.disconnect()
      figure.classList.add('wiping')
      requestAnimationFrame(() => set(100))
      setTimeout(() => {
        figure.classList.remove('wiping')
        ready()
      }, 1300)
    },
    { threshold: 0.15 },
  )
  seen.observe(figure)
}
