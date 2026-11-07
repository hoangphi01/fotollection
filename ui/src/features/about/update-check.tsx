// SPDX-License-Identifier: GPL-3.0-only
import { CircleCheck, Download, LoaderCircle, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Field, Hint } from '@/components/forms/field'
import { Button } from '@/components/ui/button'
import { commands, type UpdateCheck as Check } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

type State =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'done'; check: Check }
  | { kind: 'error'; message: string }

/**
 * "Check for updates" (FR-061), in the Settings window (FR-086), beside the check at start
 * (FR-103): the app goes online when this button is clicked, and once at start unless that is
 * turned off. A newer version opens its installer in the browser; installing it replaces this
 * one and keeps settings and projects.
 */
export function UpdateCheck() {
  const { t } = useTranslation()
  const [state, setState] = useState<State>({ kind: 'idle' })

  const check = async () => {
    setState({ kind: 'checking' })
    const result = await commands.checkForUpdate()
    setState(result.status === 'ok' ? { kind: 'done', check: result.data } : { kind: 'error', message: errorText(result.error) })
  }

  return (
    <Field>
      {state.kind === 'done' && state.check.newer ? (
        <>
          <p className="text-sm font-medium">{t('update.available', { version: state.check.latest })}</p>
          <Button size="sm" className="w-full" onClick={() => void commands.openUpdateDownload()}>
            <Download data-icon="inline-start" /> {t('update.download')}
          </Button>
          <Hint>{t('update.replaces')}</Hint>
        </>
      ) : (
        <>
          <Button size="sm" variant="outline" className="w-full" disabled={state.kind === 'checking'} onClick={() => void check()}>
            {state.kind === 'checking' ? <LoaderCircle data-icon="inline-start" className="animate-spin" /> : <RefreshCw data-icon="inline-start" />}
            {t(state.kind === 'checking' ? 'update.checking' : 'update.check')}
          </Button>
          {state.kind === 'done' && (
            <p className="flex items-center gap-1.5 text-xs text-success">
              <CircleCheck className="size-4" /> {t('update.latest')}
            </p>
          )}
          {state.kind === 'error' && <p className="text-xs text-destructive">{state.message}</p>}
          {state.kind !== 'done' && state.kind !== 'error' && <Hint>{t('update.privacy')}</Hint>}
        </>
      )}
    </Field>
  )
}
