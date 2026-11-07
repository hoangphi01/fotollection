// SPDX-License-Identifier: GPL-3.0-only
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import guideEn from '../../../../docs/USER_GUIDE.md?raw'
import guideVi from '../../../../docs/USER_GUIDE.vi.md?raw'
import { chapters, type Chapter } from '@/lib/manual'

/**
 * The guide's chapters in the app's language, split once per language. The guide is bundled
 * (offline, FR-110) in the manual's own chunk, loaded when the manual or its Settings section
 * first opens, never at start (NFR-010).
 */
export function useManualChapters(): Chapter[] {
  const { i18n } = useTranslation()
  const vi = i18n.language.startsWith('vi')
  return useMemo(() => chapters(vi ? guideVi : guideEn), [vi])
}
