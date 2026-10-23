// SPDX-License-Identifier: GPL-3.0-only
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { TooltipProvider } from '@/components/ui/tooltip'
import { followSystemTheme } from '@/lib/theme'
import { App } from './app'
import './index.css'

followSystemTheme()

const root = document.getElementById('root')
if (!root) throw new Error('missing #root element')

createRoot(root).render(
  <StrictMode>
    <TooltipProvider>
      <App />
    </TooltipProvider>
  </StrictMode>,
)
