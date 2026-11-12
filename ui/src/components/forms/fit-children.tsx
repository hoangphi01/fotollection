// SPDX-License-Identifier: GPL-3.0-only
import { Children, Fragment, isValidElement, type ReactNode } from 'react'
import { FitText } from './fit-text'

/**
 * A control's children with their words in one `FitText` each run (an icon, words, a count
 * stay in their order); a lone space between an icon and words is left as it is.
 */
export function fitChildren(children: ReactNode): ReactNode {
  const out: ReactNode[] = []
  let words: (string | number)[] = []
  const flush = () => {
    const joined = words.join('')
    if (joined.trim()) out.push(<FitText key={`fit-${out.length}`}>{joined}</FitText>)
    else if (joined) out.push(joined)
    words = []
  }
  for (const child of Children.toArray(children)) {
    if (typeof child === 'string' || typeof child === 'number') words.push(child)
    else {
      flush()
      out.push(isValidElement(child) && child.type === Fragment ? fitChildren((child.props as { children?: ReactNode }).children) : child)
    }
  }
  flush()
  return out.length === 1 ? out[0] : out
}
