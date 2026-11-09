// SPDX-License-Identifier: GPL-3.0-only
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { HintLayer } from '@/components/layout/hint-layer'
import { AppBoundary } from '@/components/layout/part-boundary'
import { commands } from '@/ipc/bindings'
import { noWeb } from '@/lib/no-web'
import { followSystemTheme } from '@/lib/theme'
import { App } from './app'
import './i18n'
import './index.css'

followSystemTheme()
noWeb()

if (import.meta.env.DEV) {
  // Surface CSP violations and failed images in the terminal (Vite forwards console output).
  document.addEventListener('securitypolicyviolation', (e) =>
    console.warn('CSP violation', e.violatedDirective, e.blockedURI),
  )
  document.addEventListener(
    'error',
    (e) => {
      if (e.target instanceof HTMLImageElement) console.warn('image failed', e.target.src)
    },
    true,
  )
}

const root = document.getElementById('root')
if (!root) throw new Error('missing #root element')

// Every error the interface meets reaches the error log through Rust (ADR-0048): a part an
// error boundary caught (`ERR-UI-0001`), and anything nothing caught (`ERR-UI-0002`).
const report = (kind: 'render' | 'unhandled', error: unknown) =>
  void commands.reportUiError(kind, error instanceof Error ? `${error.name}: ${error.message}` : String(error)).catch(() => {})
window.addEventListener('error', (e) => report('unhandled', e.error ?? e.message))
window.addEventListener('unhandledrejection', (e) => report('unhandled', e.reason))

createRoot(root, {
  onCaughtError: (error) => report('render', error),
  onUncaughtError: (error) => report('render', error),
}).render(
  <StrictMode>
    {/* The last resort: the whole app in one part, so even an error outside every panel shows
        a note with *Reload*, never a blank window. */}
    <AppBoundary>
      {/* Hints are `data-hint` and the HintLayer only (STYLE_RULE §4: one kind of tooltip). */}
      <App />
    </AppBoundary>
    <HintLayer />
  </StrictMode>,
)
