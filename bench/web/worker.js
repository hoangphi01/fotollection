// SPDX-License-Identifier: GPL-3.0-only
// Browser baseline for docs/OPTIMIZATION.md §2: the same work as Fotollection's export,
// done with the web platform (createImageBitmap + OffscreenCanvas + convertToBlob).

const RATIO = 3 / 2 // 10×15 print, long:short
const QUALITY = 0.95

function centerCrop(w, h) {
  const [rw, rh] = w >= h ? [RATIO, 1] : [1, RATIO]
  if (w * rh >= h * rw) {
    const cw = Math.round((h * rw) / rh)
    return [Math.floor((w - cw) / 2), 0, cw, h]
  }
  const ch = Math.round((w * rh) / rw)
  return [0, Math.floor((h - ch) / 2), w, ch]
}

function stamp(ctx, w, h) {
  // Same placement as the Rust stamp: ~3.6 mm text, 4 mm from the edge on a 150 mm print.
  const pxPerMm = Math.max(w, h) / 150
  ctx.font = `600 ${3.6 * pxPerMm}px sans-serif`
  ctx.textAlign = 'right'
  ctx.textBaseline = 'bottom'
  const m = 4 * pxPerMm
  ctx.fillStyle = 'rgba(0,0,0,0.55)'
  ctx.fillText('2024-09-11', w - m + pxPerMm * 0.2, h - m + pxPerMm * 0.2)
  ctx.fillStyle = '#fff'
  ctx.fillText('2024-09-11', w - m, h - m)
}

async function render(bitmap, maxLongEdge) {
  const [sx, sy, sw, sh] = centerCrop(bitmap.width, bitmap.height)
  const scale = maxLongEdge ? Math.min(1, maxLongEdge / Math.max(sw, sh)) : 1
  const [ow, oh] = [Math.round(sw * scale), Math.round(sh * scale)]
  const canvas = new OffscreenCanvas(ow, oh)
  const ctx = canvas.getContext('2d')
  ctx.imageSmoothingQuality = 'high'
  ctx.drawImage(bitmap, sx, sy, sw, sh, 0, 0, ow, oh)
  stamp(ctx, ow, oh)
  const blob = await canvas.convertToBlob({ type: 'image/jpeg', quality: QUALITY })
  return blob.size
}

self.onmessage = async ({ data }) => {
  if (data.kind === 'export') {
    // Full-resolution export of a list of photos.
    for (const url of data.urls) {
      const blob = await (await fetch(url)).blob()
      const bitmap = await createImageBitmap(blob, { imageOrientation: 'from-image' })
      await render(bitmap, null)
      bitmap.close()
      self.postMessage({ kind: 'done' })
    }
    self.postMessage({ kind: 'finished' })
  } else if (data.kind === 'preview') {
    // Interactive preview: decode once, then re-render a 1600 px preview repeatedly,
    // like Fotollection's T2 cache (OPTIMIZATION.md §4.1).
    const t0 = performance.now()
    const blob = await (await fetch(data.url)).blob()
    const full = await createImageBitmap(blob, { imageOrientation: 'from-image' })
    const scale = Math.min(1, 2048 / Math.max(full.width, full.height))
    const t2 = await createImageBitmap(full, {
      resizeWidth: Math.round(full.width * scale),
      resizeHeight: Math.round(full.height * scale),
      resizeQuality: 'high',
    })
    full.close()
    await render(t2, 1600)
    const cold = performance.now() - t0
    const warm = []
    for (let i = 0; i < data.repeats; i++) {
      const t = performance.now()
      await render(t2, 1600)
      warm.push(performance.now() - t)
    }
    self.postMessage({ kind: 'preview', cold, warm })
  }
}
