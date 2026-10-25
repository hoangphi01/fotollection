// SPDX-License-Identifier: GPL-3.0-only
import { useEffect } from 'react'
import { commands, events } from '@/ipc/bindings'

/** Never keep the workspace hidden longer than this, whatever happens. */
const SAFETY_MS = 15_000
/** Matches the fade in index.html. */
const FADE_MS = 200

/**
 * The start-up screen (FR-060): the logo and a loading bar, painted by index.html before
 * any script runs, cover the workspace while the last session comes back. The bar follows
 * the import's progress when there is one. Once Rust says start-up is over, the screen
 * fades out on the finished workspace.
 */
export function useStartupScreen() {
  useEffect(() => {
    const splash = document.getElementById('splash')
    if (!splash) return
    let finished = false
    const finish = () => {
      if (finished) return
      finished = true
      splash.classList.add('done')
      window.setTimeout(() => splash.remove(), FADE_MS + 50)
    }
    const progress = events.taskProgress.listen(({ payload }) => {
      if (payload.total === 0) return
      splash.dataset.progress = ''
      splash.style.setProperty('--splash-progress', `${Math.round((payload.done / payload.total) * 100)}%`)
    })
    // Listen first, then ask, so a start-up that ends in between is not missed.
    const started = events.startupFinished.listen(finish)
    void started.then(() => commands.startupFinished()).then((done) => done && finish())
    const safety = window.setTimeout(finish, SAFETY_MS)
    return () => {
      window.clearTimeout(safety)
      void progress.then((off) => off())
      void started.then((off) => off())
    }
  }, [])
}
