// SPDX-License-Identifier: GPL-3.0-only

/** One chapter of the user guide, as the manual inside the app shows it (FR-110). */
export type Chapter = {
  /** The anchor the site uses too (`<!-- chapter: id -->`). */
  id: string
  number: string
  title: string
  /** Its Markdown, made for the app: screenshots as images, the PRD links as words. */
  body: string
  /** The same text cut into the app's own cards (FR-110; owner, 2026-11-07: the guide gives
   * the words and pictures, the layout is the app's). */
  cards: ManualCard[]
}

/** One card of a chapter: a topic, a step or what the chapter opens with. */
export type ManualCard = {
  /** `overview` (the chapter's opening, titled by the app), `topic` (a `**Lead:**` paragraph
   * or a `###` section) or `step` (a numbered step with a bold lead). */
  kind: 'overview' | 'topic' | 'step'
  title: string
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
    if (!current) return
    current.chapter.body = current.lines.join('\n').trim()
    current.chapter.cards = cardsOf(current.chapter.body)
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
      const chapter: Chapter = { id: pending, number: title[1] ?? '', title: title[2] ?? '', body: '', cards: [] }
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

// A paragraph or step that opens with its name in bold: "**Name:** …", "**Name**: …", "**Name?** …".
const LEAD = /^\*\*([^*]{1,80}?)(?::\*\*:?|\*\*:|(\?)\*\*)\s*/
const STEP = /^(\d+)\. /

/** The chapter's top-level blocks: paragraphs, lists (with their indented lines), tables,
 * pictures and headings, each as its lines. */
function blocksOf(body: string): string[][] {
  const blocks: string[][] = []
  let block: string[] = []
  const flush = () => {
    if (block.length > 0) blocks.push(block)
    block = []
  }
  for (const line of body.split('\n')) {
    if (line.trim() === '') {
      flush()
      continue
    }
    // A new step, a heading or a picture starts its own block; a step's indented lines and
    // a list's next item stay with it.
    if (STEP.test(line) || line.startsWith('#') || line.startsWith('![')) flush()
    block.push(line)
  }
  flush()
  return blocks
}

/**
 * The chapter cut into cards: what it opens with, then a card for each `###` section, each
 * paragraph led by a bold name and each numbered step led by one ("1. **Open a folder:**"
 * becomes the card *1. Open a folder*). Everything else (lists, tables, pictures, plain
 * paragraphs) stays in the card before it.
 */
export function cardsOf(body: string): ManualCard[] {
  const cards: ManualCard[] = [{ kind: 'overview', title: '', body: '' }]
  const add = (text: string) => {
    const last = cards[cards.length - 1]
    if (last) last.body = last.body ? `${last.body}\n\n${text}` : text
  }
  for (const lines of blocksOf(body)) {
    const first = lines[0] ?? ''
    const heading = /^#{3,4} (?:[\d.]+ )?(.+)$/.exec(first)
    if (heading) {
      cards.push({ kind: 'topic', title: heading[1] ?? '', body: lines.slice(1).join('\n') })
      continue
    }
    const step = STEP.exec(first)
    if (step) {
      const text = first.slice(step[0].length)
      // The step's own lines lose the room its number took.
      const more = lines.slice(1).map((l) => l.replace(/^ {1,3}/, ''))
      const stepLead = LEAD.exec(text)
      const bold = /^\*\*([^*]{1,80})\*\*/.exec(text)
      if (stepLead) {
        const rest = text.slice(stepLead[0].length)
        cards.push({ kind: 'step', title: `${step[1]}. ${stepLead[1]}${stepLead[2] ?? ''}`, body: [rest, ...more].join('\n').trim() })
        continue
      }
      // "2. **The sidebar** lists…": named by its bold words, which stay in its sentence.
      if (bold) {
        cards.push({ kind: 'step', title: `${step[1]}. ${bold[1]}`, body: [text, ...more].join('\n').trim() })
        continue
      }
    }
    const lead = step ? null : LEAD.exec(first)
    if (lead) {
      const rest = [first.slice(lead[0].length), ...lines.slice(1)].join('\n').trim()
      cards.push({ kind: 'topic', title: `${lead[1]}${lead[2] ?? ''}`, body: rest })
      continue
    }
    add(lines.join('\n'))
  }
  return cards.filter((c) => c.body.trim() !== '' || c.kind !== 'overview')
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
