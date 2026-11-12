// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { ListRow, ListRows } from '@/components/forms/list-row'
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

  return (
    <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 rounded-lg border chrome-lit lifted">
      <p
        key={reduced ? 'still' : turn}
        lang={shown?.value}
        aria-hidden
        className="motion-in-slow animate-in fade-in text-6xl leading-[1.6] font-semibold tracking-tight"
      >
        {shown?.hello}
      </p>
      <div className="w-72 rounded-2xl border bg-background/60 p-2 backdrop-blur-xl lifted">
        <ListRows>
          {LANGUAGES.map((l) => (
            <ListRow key={l.value} chosen={l.value === language} onClick={() => void change(l.value)}>
              <span lang={l.value}>{l.label}</span>
            </ListRow>
          ))}
        </ListRows>
      </div>
    </div>
  )
}
