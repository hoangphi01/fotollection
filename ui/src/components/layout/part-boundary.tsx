// SPDX-License-Identifier: GPL-3.0-only
import { RefreshCw, TriangleAlert } from 'lucide-react'
import { Component, type ErrorInfo, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { faultAt } from '@/lib/fault'
import { PanelNote } from './panel-note'

type PartBoundaryProps = {
  /** What this part is, for the injected fault (`card`, `panel`, `dialog`, `stage`…). */
  part: string
  children: ReactNode
}

/**
 * One part of the interface failing alone (ADR-0048): an error while this part renders shows
 * `PartFailed` in its place, the rest of the window goes on; before, any render error blanked
 * the whole window. The error itself reaches the error log through the root's `onCaughtError`
 * (`main.tsx`, `ERR-UI-0001`). A new card, panel or dialog is inside one through its block
 * (`SectionCard`, `SidePanel`, `DialogContent`, `ViewerFrame`, `SettingsFrame`).
 */
export class PartBoundary extends Component<PartBoundaryProps, { failed: boolean }> {
  override state = { failed: false }

  static getDerivedStateFromError() {
    return { failed: true }
  }

  override componentDidCatch(_error: Error, _info: ErrorInfo) {
    // Logged by the root's `onCaughtError`; nothing more to do here.
  }

  override render() {
    if (this.state.failed) return <PartFailed onRetry={() => this.setState({ failed: false })} />
    return (
      <>
        <Fault part={this.props.part} />
        {this.props.children}
      </>
    )
  }
}

/** Throws only when the harness asks for this part to fail (`lib/fault.ts`). */
function Fault({ part }: { part: string }) {
  faultAt(part)
  return null
}

/** What a failed part shows: a short note and *Try again*, which renders it afresh. */
export function PartFailed({ onRetry }: { onRetry: () => void }) {
  const { t } = useTranslation()
  return (
    <div data-slot="part-failed">
      <PanelNote
        icon={<TriangleAlert />}
        action={
          <Button variant="outline" size="sm" onClick={onRetry}>
            <RefreshCw data-icon="inline-start" /> {t('part.retry')}
          </Button>
        }
      >
        {t('part.failed')}
      </PanelNote>
    </div>
  )
}

/**
 * The whole window as one part, the last boundary: an error no panel caught shows a note with
 * *Reload* on the app's ground instead of a blank window.
 */
export class AppBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  override state = { failed: false }

  static getDerivedStateFromError() {
    return { failed: true }
  }

  override render() {
    if (!this.state.failed) return this.props.children
    return <AppFailed />
  }
}

function AppFailed() {
  const { t } = useTranslation()
  return (
    <div data-slot="app-failed" className="flex h-screen items-center justify-center bg-background p-8">
      <div className="w-full max-w-md">
        <PanelNote
          icon={<TriangleAlert />}
          action={
            <Button size="sm" onClick={() => window.location.reload()}>
              <RefreshCw data-icon="inline-start" /> {t('part.reload')}
            </Button>
          }
        >
          {t('part.appFailed')}
        </PanelNote>
      </div>
    </div>
  )
}
