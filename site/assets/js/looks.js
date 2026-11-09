// SPDX-License-Identifier: GPL-3.0-only
// The landing page's Looks section (site/_includes/looks.html): a look picked fades its picture
// in over the others (all stacked, so nothing flashes while one decodes) and shows its sentence
// (all in one cell, so the row never moves); press and hold the photo for its original, as in
// the app. Only classes and attributes change: no HTML is written.
for (const root of document.querySelectorAll('[data-looks]')) {
  const stack = root.querySelector('[data-looks-stack]')
  const pictures = [...stack.querySelectorAll('img')]
  const says = [...root.querySelectorAll('.looks-say')]
  const choices = [...root.querySelectorAll('.look-choice')]
  let current = choices[0].dataset.slug
  const show = (slug) => {
    for (const img of pictures) img.classList.toggle('on', img.dataset.slug === slug)
  }
  const pick = (button) => {
    current = button.dataset.slug
    for (const b of choices) b.setAttribute('aria-pressed', String(b === button))
    for (const p of says) p.classList.toggle('on', p.dataset.slug === current)
    show(current)
  }
  for (const b of choices) b.addEventListener('click', () => pick(b))
  // Arrow keys move between the looks, as in a toolbar.
  root.querySelector('.looks-choices').addEventListener('keydown', (e) => {
    const i = choices.indexOf(document.activeElement)
    if (i < 0 || !['ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp'].includes(e.key)) return
    e.preventDefault()
    const next = choices[(i + (e.key === 'ArrowRight' || e.key === 'ArrowDown' ? 1 : choices.length - 1)) % choices.length]
    next.focus()
    pick(next)
  })
  // Press and hold for the original; let go and the look comes back. The press is captured,
  // and the browser's own image drag is off, or it would cancel the press at once.
  stack.addEventListener('pointerdown', (e) => {
    e.preventDefault()
    stack.setPointerCapture?.(e.pointerId)
    show(choices[0].dataset.slug)
  })
  for (const ev of ['pointerup', 'pointercancel', 'lostpointercapture']) stack.addEventListener(ev, () => show(current))
  stack.addEventListener('dragstart', (e) => e.preventDefault())
  stack.addEventListener('contextmenu', (e) => e.preventDefault())
  // Every picture loads once the section is near, so the first pick is already there.
  for (const img of pictures) img.loading = 'eager'
}
