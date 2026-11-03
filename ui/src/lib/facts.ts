// SPDX-License-Identifier: GPL-3.0-only
import i18n from '@/i18n'
import type { ImageFacts } from '@/ipc/bindings'
import { isMac } from '@/lib/platform'

/** Names that stay as they are in every language. */
const SPACE: Partial<Record<NonNullable<ImageFacts['space']>, string>> = {
  srgb: 'sRGB',
  displayP3: 'Display P3',
  adobeRgb: 'Adobe RGB',
  proPhoto: 'ProPhoto RGB',
  cmyk: 'CMYK',
}
const CHROMA: Partial<Record<NonNullable<ImageFacts['chroma']>, string>> = {
  rgb: 'RGB',
  cmyk: 'CMYK',
  ycck: 'YCCK',
}
const DECODER: Record<NonNullable<ImageFacts['decoder']>, string> = {
  zune: 'zune-jpeg',
  reference: 'jpeg-decoder',
  own: 'Fotollection',
  rawler: 'rawler',
  system: isMac ? 'ImageIO' : 'WIC',
}

/**
 * What a file is (FR-071), as the inspector's line: "JPEG · 4:2:0 · Progressive · 8-bit ·
 * Display P3 · read by zune-jpeg" or "ARW · Bayer RGGB · 14-bit · read by rawler". The
 * format comes from the file name; the DNG kinds the develop does not read yet follow.
 */
export function factsParts(facts: ImageFacts, fileName: string): string[] {
  const t = i18n.t
  const parts: string[] = []
  const ext = fileName.includes('.') ? fileName.split('.').pop() : undefined
  if (ext) {
    const format = ext.toUpperCase()
    parts.push(format === 'JPG' || format === 'JPE' ? 'JPEG' : format)
  }
  if (facts.chroma) {
    const c = facts.chroma
    parts.push(CHROMA[c] ?? (c === 'grey' ? t('facts.grey') : c === 'other' ? t('facts.otherSampling') : c))
  }
  if (facts.coding) parts.push(t(`facts.coding.${facts.coding}`))
  if (facts.sensor) {
    const name = t(`facts.sensor.${facts.sensor}`)
    parts.push(facts.cfa ? `${name} ${facts.cfa}` : name)
  }
  if (facts.bits != null) parts.push(t('facts.bits', { count: facts.bits }))
  if (facts.space && facts.space !== 'grey') {
    const s = facts.space
    parts.push(SPACE[s] ?? (s === 'untagged' ? t('facts.untagged') : t('facts.otherProfile')))
  }
  const dng = facts.dng
  if (dng) {
    if (dng.lossy) parts.push(t('facts.lossy'))
    if (dng.jpegXl) parts.push('JPEG XL')
    if (dng.float) parts.push(t('facts.float'))
    if (dng.gainMap) parts.push(t('facts.gainMap'))
    if (dng.originalRaw) parts.push(t('facts.originalRaw'))
  }
  if (facts.decoder) parts.push(t('facts.readBy', { name: DECODER[facts.decoder] }))
  return parts
}
