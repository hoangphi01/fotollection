// SPDX-License-Identifier: GPL-3.0-only

export type ReleaseSummary = {
  version: string
  date: string | null
  /** The first two Highlights, or the first two entries when a version has none. */
  lines: string[]
  /** Everything the version brought, by kind (Added, Changed, Fixed…), as plain text. */
  notes: { kind: string; entries: string[] }[]
}

/** Strips Markdown emphasis, code and links, so a line reads as plain text. */
function plain(line: string): string {
  return line
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/[*_`]/g, '')
    .trim()
}

/**
 * Released versions from the CHANGELOG, newest first, each with two short lines (FR-021).
 * `Unreleased` is left out.
 */
export function releaseSummaries(markdown: string): ReleaseSummary[] {
  const sections = markdown.split(/^## /m).slice(1)
  const releases: ReleaseSummary[] = []
  for (const section of sections) {
    const heading = /^\[([^\]]+)\](?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/.exec(section)
    if (!heading?.[1] || heading[1] === 'Unreleased') continue
    // Up to the next sub-heading or the end of the section (not the end of a line).
    const highlights = /^### Highlights\s*\n([\s\S]*?)(?=^### |(?![\s\S]))/m.exec(section)?.[1] ?? section
    const bullets = highlights.split('\n').filter((l) => l.startsWith('- '))
    // A version without bullets (a pre-release note) shows its first sentence instead.
    const prose = section
      .split('\n')
      .slice(1)
      .find((l) => l.trim() && !l.startsWith('#'))
    const lines = (bullets.length > 0 ? bullets.map((l) => l.slice(2)) : prose ? [prose] : [])
      .slice(0, 2)
      .map(plain)
    // The whole entry, for "Show all" in the Settings window (FR-086): each sub-heading but
    // Highlights, which the two lines above already give, with its bullets as plain text.
    const notes = section
      .split(/^### /m)
      .slice(1)
      .map((part) => {
        const [kind = '', ...rest] = part.split('\n')
        const entries = rest.filter((l) => l.startsWith('- ')).map((l) => plain(l.slice(2)))
        return { kind: kind.trim(), entries }
      })
      .filter((n) => n.kind !== 'Highlights' && n.entries.length > 0)
    releases.push({ version: heading[1], date: heading[2] ?? null, lines, notes })
  }
  return releases
}
