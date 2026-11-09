// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useState } from 'react'
import { commands } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/**
 * What start-up has to tell the user, once (ADR-0048): a damaged settings file recovered,
 * restored from its copy or left alone, never fixed in silence. Shown in the main view's
 * error notice until dismissed.
 */
export function useStartupNotice(): string | null {
  const [notice, setNotice] = useState<string | null>(null)
  useEffect(() => {
    void commands.takeStartupNotices().then((list) => {
      if (list.length > 0) setNotice(list.map(errorText).join(' '))
    })
  }, [])
  return notice
}
