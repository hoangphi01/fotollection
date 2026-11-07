// SPDX-License-Identifier: GPL-3.0-only
import { Info } from 'lucide-react'
import Markdown, { type Components } from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Hint } from '@/components/forms/field'
import { PanelNote } from '@/components/layout/panel-note'
import { Kbd } from '@/components/ui/kbd'
import { SHOT_PREFIX } from '@/lib/manual'

// Bold words that are a key or keys ("J", "⇧R", "Cmd/Ctrl+A", "← →", "Esc"): shown as keys,
// as the key hints under every viewer show them, never as bold words.
const BASE = String.raw`(?:Cmd|Ctrl|Alt|Option|Shift|Esc|Tab|Enter|Return|Space|Delete|Backspace|F\d{1,2}|[A-Z0-9←→↑↓\\\[\]{},.+−-])`
const KEYS = new RegExp(String.raw`^[⌘⇧⌥]*${BASE}(?:\s*[+/ ]\s*[⌘⇧⌥]*${BASE})*$`)

/** The plain text of a node's children, to tell a key from a bold word. */
function textOf(children: unknown): string {
  if (typeof children === 'string') return children
  if (Array.isArray(children)) return children.map(textOf).join('')
  return ''
}

/**
 * A card's words from the manual (FR-110), as React elements: raw HTML in them is skipped,
 * never rendered (SECURITY.md §7: no HTML from text). The guide gives the words and the pictures; the
 * look is the app's (owner, 2026-11-07): each block a child of its `SectionCard`, which spaces
 * them 12 px, text at the card's size, lists 4 px apart as every list (STYLE_RULE §4), keys as
 * `Kbd`, a picture framed as the welcome's with its words under it as a `Hint`. Links show as
 * their words: the app opens nothing on the web from here.
 */
export function ManualText({ markdown, picture }: { markdown: string; picture: (name: string) => string }) {
  const components: Components = {
    h3: ({ children }) => <p className="font-semibold">{children}</p>,
    h4: ({ children }) => <p className="font-semibold">{children}</p>,
    // A picture alone is its own block, not a paragraph (its words under it are a `Hint`).
    p: ({ node, children }) => {
      const only = node?.children.length === 1 ? node.children[0] : undefined
      if (only?.type === 'element' && only.tagName === 'img') return <>{children}</>
      return <p className="leading-normal">{children}</p>
    },
    ul: ({ children }) => <ul className="flex list-disc flex-col gap-1 pl-5 leading-normal marker:text-primary">{children}</ul>,
    ol: ({ children }) => <ol className="flex list-decimal flex-col gap-1 pl-5 leading-normal marker:text-primary">{children}</ol>,
    li: ({ children }) => <li className="pl-1 [&>ol]:pt-1 [&>ul]:pt-1">{children}</li>,
    strong: ({ children }) => {
      if (KEYS.test(textOf(children).trim())) return <Kbd>{children}</Kbd>
      return <strong className="font-semibold">{children}</strong>
    },
    blockquote: ({ children }) => <PanelNote icon={<Info />}>{children}</PanelNote>,
    code: ({ children }) => <code className="rounded-sm bg-muted px-1 font-mono text-xs">{children}</code>,
    a: ({ children }) => <span>{children}</span>,
    hr: () => null,
    table: ({ children }) => (
      <div className="overflow-x-auto rounded-md border">
        <table className="w-full text-xs">{children}</table>
      </div>
    ),
    th: ({ children }) => <th className="bg-muted/70 px-2 py-1 text-left font-semibold">{children}</th>,
    td: ({ children }) => <td className="border-t px-2 py-1 align-top">{children}</td>,
    img: ({ src, alt }) => {
      if (typeof src !== 'string' || !src.startsWith(SHOT_PREFIX)) return null
      // A lazy picture keeps its room before it loads (LESSONS), 1100 × 700 as most shots are.
      return (
        <div className="flex flex-col gap-1.5">
          <img
            src={picture(src.slice(SHOT_PREFIX.length))}
            alt={alt ?? ''}
            width={1100}
            height={700}
            loading="lazy"
            className="h-auto w-full rounded-md border"
          />
          <Hint>{alt}</Hint>
        </div>
      )
    },
  }
  return (
    <Markdown remarkPlugins={[remarkGfm]} components={components} skipHtml>
      {markdown}
    </Markdown>
  )
}
