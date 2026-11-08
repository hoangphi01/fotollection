// SPDX-License-Identifier: GPL-3.0-only
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { HintLayer } from '@/components/layout/hint-layer'
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

createRoot(root).render(
  <StrictMode>
    {/* Hints are `data-hint` and the HintLayer only (STYLE_RULE §4: one kind of tooltip). */}
    <App />
    <HintLayer />
  </StrictMode>,
)
