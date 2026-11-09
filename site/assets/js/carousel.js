// SPDX-License-Identifier: GPL-3.0-only
// The landing page's carousels (site/_includes/carousel.html): ‹ › scroll by one card; each is
// off at its end. Scrolling itself is the browser's (scroll-snap), so a swipe, a trackpad and
// the keyboard work without this script.
for (const root of document.querySelectorAll('[data-carousel]')) {
  const track = root.querySelector('.carousel-track')
  const prev = root.querySelector('[data-carousel-prev]')
  const next = root.querySelector('[data-carousel-next]')
  const step = () => {
    const card = track.querySelector('.carousel-card')
    const gap = parseFloat(getComputedStyle(track).columnGap) || 0
    return card ? card.getBoundingClientRect().width + gap : track.clientWidth
  }
  const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches
  const go = (dir) => track.scrollBy({ left: dir * step(), behavior: reduce ? 'auto' : 'smooth' })
  prev.addEventListener('click', () => go(-1))
  next.addEventListener('click', () => go(1))
  const update = () => {
    prev.disabled = track.scrollLeft <= 2
    next.disabled = track.scrollLeft + track.clientWidth >= track.scrollWidth - 2
  }
  track.addEventListener('scroll', () => requestAnimationFrame(update), { passive: true })
  addEventListener('resize', update)
  update()
}
