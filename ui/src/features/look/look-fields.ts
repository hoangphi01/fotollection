// SPDX-License-Identifier: GPL-3.0-only
import type { Adjust, Grain, LookSettings, Mixer, Tint } from '@/ipc/bindings'

/** A look with every field filled: Rust's defaults stand for what a file leaves out. */
export type FullAdjust = Required<Omit<Adjust, 'shadowTint' | 'highlightTint' | 'grain' | 'mixer'>> & {
  shadowTint: Required<Tint>
  highlightTint: Required<Tint>
  grain: Required<Grain>
  mixer: Required<Mixer>
}

/** The eight hues of the colour mixer, as Rust's `colour_ops::BANDS` (FR-079). */
export const MIXER_BANDS = ['red', 'orange', 'yellow', 'green', 'aqua', 'blue', 'purple', 'magenta'] as const
/** Each hue's centre on the HSV wheel, degrees, as `BANDS` in Rust. */
const MIXER_HUES = [0, 30, 60, 120, 180, 240, 270, 300] as const
export type MixerPart = 'hue' | 'sat' | 'lum'

/**
 * A mixer slider's track, as Lightroom draws it (owner, 2026-11-08): hue from the colour the
 * slider turns it toward on the left to the one on the right, saturation from grey to the full
 * colour, luminance from dark to light. A swatch of what prints (STYLE_RULE §4).
 */
export function mixerTrack(part: MixerPart, band: number): string {
  const h = MIXER_HUES[band] ?? 0
  const c = (hue: number, s: number, l: number) => `hsl(${hue} ${s}% ${l}%)`
  if (part === 'hue') return `linear-gradient(to right, ${c(h - 30, 75, 55)}, ${c(h, 75, 55)}, ${c(h + 30, 75, 55)})`
  if (part === 'sat') return `linear-gradient(to right, ${c(h, 0, 55)}, ${c(h, 85, 55)})`
  return `linear-gradient(to right, ${c(h, 70, 18)}, ${c(h, 70, 55)}, ${c(h, 70, 88)})`
}
const EIGHT = (): Required<Mixer>['hue'] => [0, 0, 0, 0, 0, 0, 0, 0]
export type FullLook = {
  adjust: FullAdjust
  lut: string | null
  lutStrength: number
  amount: number
  /** The look of the library it came from, kept while its sliders move (FR-079). */
  name: string | null
  base: string | null
}

/** A look's change, as the cards send it: `first` starts a new undo step (FR-079). */
export type LookChange = (change: (look: FullLook) => FullLook, first: boolean) => void

/** The sliders Faves shows at once, then those behind *More* (FR-079, the three levels). */
export const FAVES_FIRST = ['brightness', 'contrast', 'warmth', 'saturation'] as const
export const FAVES_MORE = ['highlights', 'shadows', 'tint', 'vibrance'] as const

/** What Tools holds, for its "· N changed" and its reset. */
const TOOLS = ['dehaze', 'fade', 'shadowTint', 'highlightTint', 'balance', 'mono'] as const

export type SignedKey = (typeof FAVES_FIRST)[number] | (typeof FAVES_MORE)[number]

/** The settings' default look: nothing set, everything at 100 %. */
export const NO_LOOK: FullLook = {
  adjust: {
    brightness: 0,
    contrast: 0,
    highlights: 0,
    shadows: 0,
    warmth: 0,
    tint: 0,
    saturation: 0,
    vibrance: 0,
    fade: 0,
    shadowTint: { hue: 0, amount: 0 },
    highlightTint: { hue: 0, amount: 0 },
    balance: 0,
    mono: 'off',
    grain: { amount: 0, size: 0 },
    vignette: 0,
    dehaze: 0,
    mixer: { hue: EIGHT(), sat: EIGHT(), lum: EIGHT() },
  },
  lut: null,
  lutStrength: 100,
  amount: 100,
  name: null,
  base: null,
}

/** `look` with Rust's defaults where it says nothing. */
export function fullLook(look: LookSettings | undefined): FullLook {
  const a = look?.adjust
  return {
    adjust: {
      ...NO_LOOK.adjust,
      ...a,
      shadowTint: { ...NO_LOOK.adjust.shadowTint, ...a?.shadowTint },
      highlightTint: { ...NO_LOOK.adjust.highlightTint, ...a?.highlightTint },
      grain: { ...NO_LOOK.adjust.grain, ...a?.grain },
      mixer: {
        hue: a?.mixer?.hue ?? EIGHT(),
        sat: a?.mixer?.sat ?? EIGHT(),
        lum: a?.mixer?.lum ?? EIGHT(),
      },
    },
    lut: look?.lut ?? null,
    lutStrength: look?.lutStrength ?? 100,
    amount: look?.amount ?? 100,
    name: look?.name ?? null,
    base: look?.base ?? null,
  }
}

const isSet = (adjust: FullAdjust, key: (typeof TOOLS)[number] | SignedKey): boolean => {
  const v = adjust[key]
  if (typeof v === 'number') return v !== 0
  if (typeof v === 'string') return v !== 'off'
  return v.amount !== 0
}

export const favesChanged = (a: FullAdjust) => [...FAVES_FIRST, ...FAVES_MORE].filter((k) => isSet(a, k)).length
export const toolsChanged = (a: FullAdjust) => TOOLS.filter((k) => isSet(a, k)).length

export const resetFaves = (a: FullAdjust): FullAdjust => ({
  ...a,
  ...Object.fromEntries([...FAVES_FIRST, ...FAVES_MORE].map((k) => [k, 0])),
})

export const resetTools = (a: FullAdjust): FullAdjust => ({
  ...a,
  dehaze: 0,
  fade: 0,
  shadowTint: NO_LOOK.adjust.shadowTint,
  highlightTint: NO_LOOK.adjust.highlightTint,
  balance: 0,
  mono: 'off',
})

/** A colour of the wheel as the shadow and highlight tints use it (OKLCh, as in Rust). */
export const hueSwatch = (hue: number) => `oklch(0.72 0.12 ${hue})`

/** How many of the mixer's 24 sliders are moved, for its "· N changed" (FR-079). */
export const mixerChanged = (a: FullAdjust) => [...a.mixer.hue, ...a.mixer.sat, ...a.mixer.lum].filter((v) => v !== 0).length
