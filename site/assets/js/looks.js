// SPDX-License-Identifier: GPL-3.0-only
// The landing page's Looks section (site/_includes/looks.html): a look picked shows the photo
// through it and its sentence; press and hold the photo for its original, as in the app.
// Text is set with textContent only, never as HTML.
for (const root of document.querySelectorAll('[data-looks]')) {
  const photo = root.querySelector('[data-looks-photo]')
  const name = root.querySelector('[data-looks-name]')
  const text = root.querySelector('[data-looks-text]')
  const choices = [...root.querySelectorAll('.look-choice')]
  let current = photo.getAttribute('src')
  const pick = (button) => {
    for (const b of choices) b.setAttribute('aria-pressed', String(b === button))
    current = button.dataset.look
    photo.src = current
    name.textContent = `${button.textContent}.`
    text.textContent = button.dataset.say
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
  // Press and hold for the original; let go and the look comes back.
  const hold = () => { photo.src = photo.dataset.original }
  const release = () => { photo.src = current }
  photo.addEventListener('pointerdown', hold)
  for (const ev of ['pointerup', 'pointerleave', 'pointercancel']) photo.addEventListener(ev, release)
  photo.addEventListener('contextmenu', (e) => e.preventDefault())
  // The other looks load once the page is idle, so a pick shows at once.
  const warm = () => choices.forEach((b) => { new Image().src = b.dataset.look })
  ;(window.requestIdleCallback ?? setTimeout)(warm)
}
