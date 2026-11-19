// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { ListRow, ListRows } from '@/components/forms/list-row'
import { HomeZoneSetting } from '@/features/settings/home-zone-setting'
import { useLanguage } from '@/features/settings/use-language'
import { LANGUAGES } from '@/i18n'
import { useReducedMotion } from '@/lib/motion'

/** How long each greeting stays before the next (a dwell, not a motion pace). */
const GREETING_MS = 2400

/**
 * The welcome's first page (FR-124, owner, 2026-11-12: "pick the language as an iPhone's
 * welcome"): a greeting that passes through every language, each in its own word, and the
 * languages by their own names; a click speaks that language at once, menus included.
 * Reduced motion keeps the greeting of the chosen language, still.
 */
export function LanguagePage() {
  const { language, change } = useLanguage()
  const reduced = useReducedMotion()
  const [turn, setTurn] = useState(0)
  useEffect(() => {
    if (reduced) return
    const timer = window.setInterval(() => setTurn((n) => n + 1), GREETING_MS)
    return () => window.clearInterval(timer)
  }, [reduced])
  const chosen = LANGUAGES.findIndex((l) => l.value === language)
  const shown = LANGUAGES[reduced ? Math.max(chosen, 0) : turn % LANGUAGES.length] ?? LANGUAGES[0]

  const backdrop = (LANGUAGES[Math.max(chosen, 0)] ?? LANGUAGES[0])?.backdrop ?? 'en'

  return (
    <div className="relative flex min-h-0 flex-1 flex-col items-center justify-center gap-3 overflow-hidden rounded-lg border lifted">
      {/* A photo of the chosen language's country behind the greeting, softened so the words
          stay first (owner, 2026-11-15), as the other pages' pictures under frosted glass. */}
      <img
        key={backdrop}
        src={`/welcome/language-${backdrop}.webp`}
        alt=""
        aria-hidden
        className="motion-in-slow animate-in fade-in absolute inset-0 size-full scale-105 object-cover blur-sm brightness-75"
      />
      <p
        key={reduced ? 'still' : turn}
        lang={shown?.value}
        aria-hidden
        className="motion-in-slow animate-in fade-in relative text-6xl leading-[1.6] font-semibold tracking-tight text-stage-ink drop-shadow-lg"
      >
        {shown?.hello}
      </p>
      {/* Liquid glass (owner, 2026-11-15: "the blur a little lighter"): the photo shows through, its colours kept. */}
      <div className="relative w-72 rounded-2xl border bg-background/40 p-2 backdrop-blur-lg backdrop-saturate-150 lifted">
        <ListRows>
          {LANGUAGES.map((l) => (
            <ListRow
              key={l.value}
              icon={<img src={l.flag} alt="" width={18} height={18} className="rounded-full" />}
              chosen={l.value === language}
              onClick={() => void change(l.value)}
            >
              <span lang={l.value}>{l.label}</span>
            </ListRow>
          ))}
        </ListRows>
      </div>
      {/* The home time (FR-145): the computer's, or a place's. */}
      <div className="relative w-72 rounded-2xl border bg-background/40 p-3 backdrop-blur-lg backdrop-saturate-150 lifted">
        <HomeZoneSetting />
      </div>
    </div>
  )
}
