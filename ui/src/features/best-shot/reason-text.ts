// SPDX-License-Identifier: GPL-3.0-only
import type { Reason } from '@/ipc/bindings'
import i18n from '@/i18n'
import { formatNumber } from '@/lib/format'

const percent = (share: number | null) => `${formatNumber((share ?? 0) * 100, 1)} %`

/** Why the best photo won, in the interface language (FR-041, FR-020). */
export function reasonText(reason: Reason): string {
  switch (reason.kind) {
    case 'sharper':
      return i18n.t('bestShot.reason.sharper', { percent: formatNumber(reason.percent ?? 0, 0) })
    case 'fewerHighlights':
      return i18n.t('bestShot.reason.fewerHighlights', {
        best: percent(reason.best),
        runnerUp: percent(reason.runnerUp),
      })
    case 'steadierShot':
      return i18n.t('bestShot.reason.steadierShot')
    case 'eyesOpen':
      return i18n.t('bestShot.reason.eyesOpen')
    case 'betterExposure':
      return i18n.t('bestShot.reason.betterExposure')
    case 'lessNoise':
      return i18n.t('bestShot.reason.lessNoise')
  }
}
