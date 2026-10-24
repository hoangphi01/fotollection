// SPDX-License-Identifier: GPL-3.0-only
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Copy, Minus, Square, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands } from '@/ipc/bindings'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { cn } from '@/lib/utils'

type TitleBarProps = {
  /** The open document's name, shown after the app's name. */
  name: string
}

/**
 * The Windows title bar, drawn in the app's own colours like VS Code's, instead of the
 * system's white strip (FR-052). macOS keeps its native title bar. It sits above dialog
 * overlays, so the window can always be moved, resized or closed.
 */
export function TitleBar({ name }: TitleBarProps) {
  const { t } = useTranslation()
  const info = useIpcQuery(commands.appInfo)
  const windows = info?.os === 'windows'
  const [maximized, setMaximized] = useState(false)

  useEffect(() => {
    if (!windows) return
    const win = getCurrentWindow()
    const sync = () => void win.isMaximized().then(setMaximized)
    sync()
    const unlisten = win.onResized(sync)
    return () => void unlisten.then((stop) => stop())
  }, [windows])

  if (!windows) return null
  const win = getCurrentWindow()
  const control = 'inline-flex h-full w-[46px] items-center justify-center text-sidebar-foreground/80 transition-colors'

  return (
    <div
      data-tauri-drag-region
      className="relative z-[60] flex h-8 shrink-0 items-center border-b bg-sidebar text-sidebar-foreground select-none"
    >
      <img
        src="/brand/mark-28.png"
        srcSet="/brand/mark-28.png 1x, /brand/mark-56.png 2x"
        width={16}
        height={16}
        alt=""
        className="pointer-events-none ml-3 rounded-[3px]"
      />
      <span data-tauri-drag-region className="ml-2 truncate text-xs text-muted-foreground">
        Fotollection — {name}
      </span>
      <div className="ml-auto flex h-full">
        <button
          type="button"
          aria-label={t('window.minimize')}
          data-hint={t('window.minimize')}
          onClick={() => void win.minimize()}
          className={cn(control, 'hover:bg-foreground/10')}
        >
          <Minus className="size-4" />
        </button>
        <button
          type="button"
          aria-label={t(maximized ? 'window.restore' : 'window.maximize')}
          data-hint={t(maximized ? 'window.restore' : 'window.maximize')}
          onClick={() => void win.toggleMaximize()}
          className={cn(control, 'hover:bg-foreground/10')}
        >
          {maximized ? <Copy className="size-3.5 -scale-x-100" /> : <Square className="size-3.5" />}
        </button>
        <button
          type="button"
          aria-label={t('window.close')}
          data-hint={t('window.close')}
          onClick={() => void win.close()}
          className={cn(control, 'hover:bg-destructive hover:text-white')}
        >
          <X className="size-4" />
        </button>
      </div>
    </div>
  )
}
