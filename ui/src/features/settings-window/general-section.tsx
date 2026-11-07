// SPDX-License-Identifier: GPL-3.0-only
// @shown settings: in the Settings window only (STYLE_RULE §4, checked by check-lego).
import { Monitor, Moon, Sparkles, Sun, SlidersHorizontal } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import { showWelcome } from '@/features/welcome/welcome-store'
import { LanguageSetting } from '@/features/about/language-setting'
import { useUiSettings } from '@/features/settings/use-ui-settings'
import type { PreviewLayout, ThemePreference } from '@/ipc/bindings'

/** The Settings window's *General* section (FR-086): the interface's language and theme. */
export function GeneralSection() {
  const { t } = useTranslation()
  const { ui, update } = useUiSettings()
  const themes = [
    {
      value: 'system',
      label: <Monitor aria-hidden />,
      ariaLabel: t('theme.system'),
      hint: t('theme.system'),
    },
    {
      value: 'light',
      label: <Sun aria-hidden />,
      ariaLabel: t('theme.light'),
      hint: t('theme.light'),
    },
    {
      value: 'dark',
      label: <Moon aria-hidden />,
      ariaLabel: t('theme.dark'),
      hint: t('theme.dark'),
    },
  ] as const
  return (
    <SectionCard icon={<SlidersHorizontal />} title={t('settingsWindow.interface')}>
      <LanguageSetting />
      <Field label={t('theme.label')} hint={t(`theme.${ui.theme}`)}>
        <SegmentedToggle<ThemePreference>
          value={ui.theme}
          options={themes}
          onChange={(theme) => void update({ theme })}
          ariaLabel={t('theme.label')}
        />
      </Field>
      {/* The develop view's layout (FR-095); F switches it in the preview too. */}
      <Field label={t('settingsWindow.previewLayout')} hint={t('settingsWindow.previewLayoutHint')}>
        <SegmentedToggle<PreviewLayout>
          value={ui.previewLayout}
          options={[
            { value: 'dialog', label: t('settingsWindow.previewDialog') },
            { value: 'window', label: t('settingsWindow.previewWindow') },
          ]}
          onChange={(previewLayout) => void update({ previewLayout })}
          ariaLabel={t('settingsWindow.previewLayout')}
        />
      </Field>
      {/* The welcome's five pages, again (FR-109). */}
      <Field label={t('welcome.title')} hint={t('welcome.againHint')}>
        <Button variant="outline" size="sm" onClick={showWelcome}>
          <Sparkles data-icon="inline-start" /> {t('welcome.again')}
        </Button>
      </Field>
    </SectionCard>
  )
}
