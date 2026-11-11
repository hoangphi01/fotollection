// SPDX-License-Identifier: GPL-3.0-only
// The landing page's keynote video (site/_includes/keynote.html, site/README.md). Nothing is
// fetched until the video comes into view: then one file, the cut for this screen (the phone's
// below 640 px) in the first codec the browser plays (AV1, else H.264). It plays while at
// least a third of it is seen and the tab is visible, loops, and the page's own headlines follow
// its time. A button pauses and plays it (WCAG 2.2.2: anything moving for over 5 s can be
// stopped). With reduced motion it never starts by itself: the poster and the last line stand
// until the button is pressed. Without script the poster and the first line stand.
for (const figure of document.querySelectorAll('[data-keynote]')) {
  const video = figure.querySelector('video')
  const button = figure.querySelector('.keynote-toggle')
  const lines = [...figure.querySelectorAll('.keynote-line')].map((e) => ({
    e,
    from: Number(e.dataset.from),
    to: Number(e.dataset.to),
  }))
  const phone = matchMedia('(max-width: 640px)').matches
  const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches
  const cut = phone ? 'mobile' : 'desktop'
  if (phone) video.poster = video.dataset.posterMobile
  figure.classList.add('keynote-live')

  // The words follow the video's clock: each line fades in over 0.35 s and out over 0.25 s,
  // as in keynote.html, so the review video and the page agree.
  const clamp = (x) => Math.min(1, Math.max(0, x))
  const show = (t) => {
    for (const l of lines) {
      const o = Math.min(clamp((t - l.from) / 0.35), 1 - clamp((t - (l.to - 0.25)) / 0.25))
      l.e.style.opacity = String(o)
      l.e.style.transform = `translateY(${(1 - clamp((t - l.from) / 0.5)) * 14}px)`
    }
  }
  // With reduced motion the closing line stands, the one that says it all.
  show(reduce ? lines.at(-1).from + 1 : lines[0].from + 1)

  let frame = 0
  const tick = () => {
    show(video.currentTime)
    frame = requestAnimationFrame(tick)
  }
  video.addEventListener('play', () => {
    cancelAnimationFrame(frame)
    frame = requestAnimationFrame(tick)
  })
  video.addEventListener('pause', () => cancelAnimationFrame(frame))

  // One file, chosen once: the first the browser says it plays.
  let loaded = false
  const load = () => {
    if (loaded) return
    loaded = true
    const kinds = [
      ['av1', 'video/mp4; codecs="av01.0.05M.08"'],
      ['h264', 'video/mp4; codecs="avc1.640032"'],
    ]
    const [kind] = kinds.find(([, type]) => video.canPlayType(type) !== '') ?? kinds[1]
    video.src = `${figure.dataset.src}keynote-${cut}-${kind}.mp4`
  }

  let wanted = !reduce
  let seen = false
  const label = () => {
    const playing = !video.paused
    button.setAttribute('aria-label', playing ? button.dataset.pause : button.dataset.play)
    button.classList.toggle('playing', playing)
  }
  const update = () => {
    if (wanted && seen && !document.hidden) {
      load()
      video.play().catch(() => {}).finally(label)
    } else {
      video.pause()
      label()
    }
  }

  button.hidden = false
  button.addEventListener('click', () => {
    wanted = video.paused
    if (wanted) seen = true
    update()
  })
  video.addEventListener('playing', label)
  video.addEventListener('pause', label)
  document.addEventListener('visibilitychange', update)
  new IntersectionObserver(
    (entries) => {
      seen = entries.some((e) => e.isIntersecting)
      update()
    },
    { threshold: 0.35 },
  ).observe(video)
}
