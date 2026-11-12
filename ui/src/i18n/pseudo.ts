// SPDX-License-Identifier: GPL-3.0-only
// The pseudo-locale `en-XA` (FR-125, docs/LOCALIZATION.md §5): English made about 40 % longer, as German
// often is, with the marks Hungarian and Vietnamese draw, to see which controls a long
// translation will shrink or cut before one exists. Only a `VITE_PSEUDO=1` build carries it;
// `site/screenshots/shoot.mjs` imports this file to name the buttons it clicks.

const MARKS: Record<string, string> = { a: 'á', e: 'ë', i: 'í', o: 'ő', u: 'ű', A: 'Á', E: 'Ë', I: 'Í', O: 'Ő', U: 'Ű' }
const HOLE = /(\{\{[^}]+\}\})/

/** One string, longer by 40 % and marked; its placeholders kept as they are. */
export function pseudoText(text: string): string {
  const parts = text.split(HOLE)
  const letters = parts.filter((p) => !HOLE.test(p)).join('').replace(/[^\p{L}]/gu, '')
  const marked = parts.map((p) => (HOLE.test(p) ? p : p.replace(/[aeiouAEIOU]/g, (c) => MARKS[c] ?? c))).join('')
  const extra = Math.ceil(letters.length * 0.4)
  if (extra === 0) return marked
  // Joined to the last word, as a German compound: the hardest case, a word that cannot wrap.
  const pad = letters.toLowerCase().repeat(Math.ceil(extra / Math.max(letters.length, 1)) + 1).slice(0, extra)
  return marked.replace(/(\p{L})(?!.*\p{L})/u, `$1${pad}`)
}

type Tree = { [key: string]: string | Tree }

/** A whole locale, its keys as English's. */
export function pseudo(tree: Tree): Tree {
  const out: Tree = {}
  for (const [k, v] of Object.entries(tree)) out[k] = typeof v === 'string' ? pseudoText(v) : pseudo(v)
  return out
}
