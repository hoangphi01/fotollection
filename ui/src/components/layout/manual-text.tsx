// SPDX-License-Identifier: GPL-3.0-only
import Markdown, { type Components } from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { SHOT_PREFIX } from '@/lib/manual'

/**
 * A chapter of the manual inside the app (FR-110), from its Markdown, as React elements: raw
 * HTML in it is skipped, never rendered (SECURITY: no `innerHTML`). Links become their words,
 * as the app opens nothing on the web from here; a screenshot is the app's own picture, so it
 * shows offline. The blocks are spaced by their column, 12 px, as cards are (STYLE_RULE §4).
 */
export function ManualText({ markdown, picture }: { markdown: string; picture: (name: string) => string }) {
  const components: Components = {
    h3: ({ children }) => <h3 className="pt-3 text-base font-semibold">{children}</h3>,
    h4: ({ children }) => <h4 className="pt-2 text-sm font-semibold">{children}</h4>,
    p: ({ children }) => <p className="text-sm leading-relaxed">{children}</p>,
    ul: ({ children }) => <ul className="flex list-disc flex-col gap-1 pl-5 text-sm leading-relaxed">{children}</ul>,
    ol: ({ children }) => <ol className="flex list-decimal flex-col gap-1.5 pl-5 text-sm leading-relaxed">{children}</ol>,
    li: ({ children }) => <li className="pl-1 [&>ol]:pt-1 [&>ul]:pt-1">{children}</li>,
    blockquote: ({ children }) => <blockquote className="border-l-2 pl-3 text-muted-foreground">{children}</blockquote>,
    code: ({ children }) => <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs">{children}</code>,
    a: ({ children }) => <span>{children}</span>,
    hr: () => <hr className="border-border" />,
    table: ({ children }) => (
      <div className="overflow-x-auto rounded-md border">
        <table className="w-full text-sm">{children}</table>
      </div>
    ),
    th: ({ children }) => <th className="bg-muted/50 px-3 py-1.5 text-left font-medium">{children}</th>,
    td: ({ children }) => <td className="border-t px-3 py-1.5 align-top">{children}</td>,
    img: ({ src, alt }) => {
      if (typeof src !== 'string' || !src.startsWith(SHOT_PREFIX)) return null
      // A lazy picture keeps its room before it loads (LESSONS), 1100 × 700 as most shots are.
      return (
        <figure className="flex flex-col gap-1.5">
          <img
            src={picture(src.slice(SHOT_PREFIX.length))}
            alt={alt ?? ''}
            width={1100}
            height={700}
            loading="lazy"
            className="h-auto w-full rounded-md border lifted"
          />
          <figcaption className="text-xs text-muted-foreground">{alt}</figcaption>
        </figure>
      )
    },
  }
  return (
    <div className="flex flex-col gap-3">
      <Markdown remarkPlugins={[remarkGfm]} components={components} skipHtml>
        {markdown}
      </Markdown>
    </div>
  )
}
