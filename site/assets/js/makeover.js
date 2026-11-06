// SPDX-License-Identifier: GPL-3.0-only
// The landing page's makeover (owner, 2026-11-06): v0.12's gallery is wiped into v0.13's once
// it comes into view, then a handle (a range input, keyboard reachable) compares the two.
// With reduced motion the new one stands at once; without script nothing moves.
for (const figure of document.querySelectorAll('[data-makeover]')) {
  const range = figure.querySelector('.makeover-range')
  const set = (pos) => figure.style.setProperty('--pos', `${pos}%`)
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches
  const ready = () => {
    range.hidden = false
    figure.style.setProperty('--handle', '1')
    range.addEventListener('input', () => set(range.value))
  }
  if (still) {
    ready()
    continue
  }
  set(0)
  const seen = new IntersectionObserver(([entry]) => {
    if (!entry.isIntersecting) return
    seen.disconnect()
    figure.classList.add('wiping')
    requestAnimationFrame(() => set(100))
    setTimeout(() => {
      figure.classList.remove('wiping')
      ready()
    }, 1300)
  }, { threshold: 0.15 })
  seen.observe(figure)
}
