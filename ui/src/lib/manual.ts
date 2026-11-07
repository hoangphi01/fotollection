// SPDX-License-Identifier: GPL-3.0-only

/** One chapter of the user guide, as the manual inside the app shows it (FR-110). */
export type Chapter = {
  /** The anchor the site uses too (`<!-- chapter: id -->`). */
  id: string
  number: string
  title: string
  /** Its Markdown, made for the app: screenshots as images, the PRD links as words. */
  body: string
}

/** Where a screenshot marker points: the app's picture of that name (`ui/public/manual/`). */
export const SHOT_PREFIX = 'manual-shot/'

const CHAPTER = /^<!-- chapter: ([a-z0-9-]+) -->$/
const TITLE = /^## (\d+)\. (.+)$/
const SHOT = /^<!-- shot: ([a-z0-9-]+) \| (.+) -->$/
// "([FR-001](PRD.md#…))", "([FR-020](…), [FR-052](…))": the reasons behind a step, for the
// people who build the app; the app's reader gets the step alone.
const SOURCES = / ?\((?:\[[^\]]+\]\([^)\s]*\.md[^)\s]*\)(?:, | and | và |; )?)+\)/g

/** A guide line as the app shows it. */
function appLine(line: string): string {
  const shot = SHOT.exec(line)
  if (shot) return `![${shot[2] ?? ''}](${SHOT_PREFIX}${shot[1] ?? ''})`
  return line.replace(SOURCES, '')
}

/**
 * The user guide's chapters (`docs/USER_GUIDE[.vi].md`, bundled at build time so they read
 * offline), split as the website splits them (`scripts/site-manual.sh`, ADR-0031): a chapter
 * marker names the next `## N. Title`; everything before the first chapter (the header for the
 * people who write the guide) is left out.
 */
export function chapters(source: string): Chapter[] {
  const out: Chapter[] = []
  let pending: string | null = null
  let current: { chapter: Chapter; lines: string[] } | null = null
  const close = () => {
    if (current) current.chapter.body = current.lines.join('\n').trim()
  }
  for (const line of source.split('\n')) {
    const marker = CHAPTER.exec(line)
    if (marker) {
      pending = marker[1] ?? null
      continue
    }
    const title = pending === null ? null : TITLE.exec(line)
    if (title && pending !== null) {
      close()
      const chapter: Chapter = { id: pending, number: title[1] ?? '', title: title[2] ?? '', body: '' }
      current = { chapter, lines: [] }
      out.push(chapter)
      pending = null
      continue
    }
    current?.lines.push(appLine(line))
  }
  close()
  return out
}

/** The chapters whose title or text holds every word of `query`, case and accents aside. */
export function findChapters(list: readonly Chapter[], query: string): Chapter[] {
  const fold = (s: string) =>
    s
      .normalize('NFD')
      .replace(/\p{M}/gu, '')
      .replace(/đ/g, 'd')
      .replace(/Đ/g, 'D')
      .toLowerCase()
  const words = fold(query).split(/\s+/).filter(Boolean)
  if (words.length === 0) return [...list]
  return list.filter((c) => {
    const text = fold(`${c.number}. ${c.title}\n${c.body}`)
    return words.every((w) => text.includes(w))
  })
}
