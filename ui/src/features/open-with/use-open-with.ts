// SPDX-License-Identifier: GPL-3.0-only
import { useEffect, useEffectEvent } from 'react'
import { commands, events, type OpenPhoto } from '@/ipc/bindings'

/**
 * "Open with" (FR-054): calls `onRequest` with each photo the operating system asked to
 * show. Rust keeps the request until it is taken, so it is asked for when `OpenPhoto`
 * arrives and once start-up is over, in case the request came before this page listened.
 */
export function useOpenWith(onRequest: (request: OpenPhoto) => void) {
  const handle = useEffectEvent(onRequest)
  useEffect(() => {
    const take = () => void commands.takePhotoRequest().then((request) => request && handle(request))
    const opened = events.openPhoto.listen(take)
    const started = events.startupFinished.listen(take)
    take()
    return () => {
      void opened.then((off) => off())
      void started.then((off) => off())
    }
  }, [])
}
