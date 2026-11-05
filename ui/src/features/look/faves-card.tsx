// SPDX-License-Identifier: GPL-3.0-only
import { ChevronDown, RotateCcw, SunMedium } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { SliderField } from '@/components/forms/slider-field'
import { Collapse } from '@/components/layout/collapse'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import type { LookSettings } from '@/ipc/bindings'
import { FAVES_FIRST, FAVES_MORE, favesChanged, fullLook, resetFaves, type LookChange, type SignedKey } from './look-fields'
import { LookCardTitle } from './look-card-title'

/** Whether *More* is open, kept while the app runs, in every panel alike. */
let moreOpen = false

type FavesCardProps = {
  look: LookSettings | undefined
  onChange: LookChange
  /** Extra controls under the sliders: the preview's *Apply to all*. */
  footer?: ReactNode
}

/**
 * The everyday sliders (FR-079, Faves): brightness, contrast, warmth and saturation at once,
 * highlights, shadows, tint and vibrance behind *More*, so a newcomer sees four.
 */
export function FavesCard({ look, onChange, footer }: FavesCardProps) {
  const { t } = useTranslation()
  const [more, setMore] = useState(moreOpen)
  const adjust = fullLook(look).adjust
  const changed = favesChanged(adjust)
  const slider = (key: SignedKey) => (
    <SliderField
      key={key}
      label={t(`look.${key}`)}
      hint={t(`look.hints.${key}`)}
      value={adjust[key]}
      onChange={(v, first) => onChange((l) => ({ ...l, adjust: { ...l.adjust, [key]: v } }), first)}
    />
  )
  return (
    <SectionCard
      icon={<SunMedium />}
      title={<LookCardTitle title={t('look.faves')} changed={changed} />}
      action={
        changed > 0 && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={t('look.reset')}
            data-hint={t('look.reset')}
            onClick={() => onChange((l) => ({ ...l, adjust: resetFaves(l.adjust) }), true)}
          >
            <RotateCcw />
          </Button>
        )
      }
      footer={footer}
    >
      {FAVES_FIRST.map(slider)}
      <Collapse open={more} stack="card">
        {FAVES_MORE.map(slider)}
      </Collapse>
      <Button
        variant="ghost"
        size="sm"
        className="w-full"
        aria-expanded={more}
        onClick={() => {
          moreOpen = !more
          setMore(!more)
        }}
      >
        <ChevronDown data-icon="inline-start" className={more ? 'rotate-180' : undefined} />
        {t(more ? 'look.less' : 'look.more')}
      </Button>
    </SectionCard>
  )
}
