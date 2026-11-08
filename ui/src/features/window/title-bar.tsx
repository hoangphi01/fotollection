// SPDX-License-Identifier: GPL-3.0-only
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Copy, Minus, Square, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands } from '@/ipc/bindings'
import { useIpcQuery } from '@/features/settings/use-ipc-query'
import { cn } from '@/lib/utils'

/** The macOS strip: the traffic lights' own height, so they sit centred in it. */
const MAC_BAR = '1.75rem'

type TitleBarProps = {
  /** The open document's name, shown after the app's name. */
  name: string
}

/**
 * The title bar in the app's own colours, as VS Code's (FR-052): on Windows instead of the
 * system's white strip, with its own buttons; on macOS a strip under the system's traffic
 * lights, the window's title bar being transparent (`titleBarStyle: Overlay`,
 * `tauri.macos.conf.json`; owner, 2026-11-06), so the bar and the app are one surface. It sits
 * above dialog overlays and takes the pointer back from a modal (`pointer-events-auto`: Radix
 * sets `none` on <body>), and dialogs let a press on it through (`data-window-chrome`), so
 * the window can always be moved, resized or closed (owner, 2026-11-08).
 */
export function TitleBar({ name }: TitleBarProps) {
  const { t } = useTranslation()
  const info = useIpcQuery(commands.appInfo)
  const windows = info?.os === 'windows'
  const mac = info?.os === 'macos'
  const [maximized, setMaximized] = useState(false)

  // Its height, for what fills the window under it (the preview's full-window layout, FR-095).
  useEffect(() => {
    if (!windows && !mac) return
    document.documentElement.style.setProperty('--titlebar', mac ? MAC_BAR : '2rem')
    return () => {
      document.documentElement.style.removeProperty('--titlebar')
    }
  }, [windows, mac])

  useEffect(() => {
    if (!windows) return
    const win = getCurrentWindow()
    const sync = () => void win.isMaximized().then(setMaximized)
    sync()
    const unlisten = win.onResized(sync)
    return () => void unlisten.then((stop) => stop())
  }, [windows])

  if (mac)
    return (
      <div
        data-tauri-drag-region
        data-window-chrome
        className="pointer-events-auto relative z-[60] flex h-(--titlebar) shrink-0 items-center justify-center border-b bg-sidebar text-sidebar-foreground select-none"
      >
        <span data-tauri-drag-region className="truncate px-24 text-xs text-muted-foreground">
          Fotollection — {name}
        </span>
      </div>
    )
  if (!windows) return null
  const win = getCurrentWindow()
  const control = 'inline-flex h-full w-[46px] items-center justify-center text-sidebar-foreground/80 motion-control'

  return (
    <div
      data-tauri-drag-region
      data-window-chrome
      className="pointer-events-auto relative z-[60] flex h-8 shrink-0 items-center border-b bg-sidebar text-sidebar-foreground select-none"
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
