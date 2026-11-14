// SPDX-License-Identifier: GPL-3.0-only
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import guideEn from '../../../../docs/USER_GUIDE.md?raw'
import guideVi from '../../../../docs/USER_GUIDE.vi.md?raw'
import guideDe from '../../../../site/_includes/manual-short-de.md?raw'
import guideHu from '../../../../site/_includes/manual-short-hu.md?raw'
import guideJa from '../../../../site/_includes/manual-short-ja.md?raw'
import guideKo from '../../../../site/_includes/manual-short-ko.md?raw'
import { languageInfo } from '@/i18n'
import { chapters, type Chapter } from '@/lib/manual'

/** The guide of each language (docs/LOCALIZATION.md §1): the full one, or the website's short one. */
const GUIDES: Record<string, string> = { en: guideEn, vi: guideVi, de: guideDe, hu: guideHu, ja: guideJa, ko: guideKo }

/**
 * The guide's chapters in the app's language (English for a language without one), split once per language. The guide is bundled
 * (offline, FR-110) in the manual's own chunk, loaded when the manual or its Settings section
 * first opens, never at start (NFR-010).
 */
export function useManualChapters(): Chapter[] {
  const { i18n } = useTranslation()
  const code = languageInfo(i18n.language).value
  return useMemo(() => chapters(GUIDES[code] ?? guideEn), [code])
}
