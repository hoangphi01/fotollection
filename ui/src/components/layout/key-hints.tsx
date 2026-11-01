// SPDX-License-Identifier: GPL-3.0-only
import { Fragment } from 'react'
import { Kbd } from '@/components/ui/kbd'

/** One hint: the keys, then what they do. A hint with no keys is plain text. */
export type KeyHint = { keys?: string[]; label: string }

type KeyHintsProps = {
  hints: KeyHint[]
}

/**
 * The keyboard hint line at the foot of the preview, the capture screen and the Simple and
 * Collage views: the same keys, set the same way everywhere (STYLE_RULE §4).
 */
export function KeyHints({ hints }: KeyHintsProps) {
  return (
    <>
      {hints.map((hint, i) => (
        <Fragment key={hint.label}>
          {i > 0 && ' · '}
          {hint.keys?.map((key) => (
            <Fragment key={key}>
              <Kbd className="h-4 min-w-4 bg-transparent px-0.5 font-mono text-[11px]">{key}</Kbd>{' '}
            </Fragment>
          ))}
          {hint.label}
        </Fragment>
      ))}
    </>
  )
}
