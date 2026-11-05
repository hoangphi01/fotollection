// SPDX-License-Identifier: GPL-3.0-only
import { Aperture, CopyCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Field } from '@/components/forms/field'
import { SegmentedToggle } from '@/components/forms/segmented-toggle'
import { SectionCard } from '@/components/layout/section-card'
import { Button } from '@/components/ui/button'
import type { RawLook } from '@/ipc/bindings'

type RawLookCardProps = {
  value: RawLook
  onChange(look: RawLook): void
  /** Every RAW photo of the library finished this way, one undoable step. */
  onApplyToAll(): void
}

/**
 * How a camera RAW is finished (FR-072, ADR-0040), first of a RAW photo's colour cards
 * wherever they are (the preview, the Collage panel, the sheet's preview): neutral, the
 * sensor's colours with the camera's exposure, or like the JPEG the camera made. The grid's
 * thumbnails stay the camera's own preview.
 */
export function RawLookCard({ value, onChange, onApplyToAll }: RawLookCardProps) {
  const { t } = useTranslation()
  return (
    <SectionCard
      icon={<Aperture className="size-4" />}
      title={t('rawLook.title')}
      footer={
        <Button variant="outline" size="sm" className="w-full" onClick={onApplyToAll}>
          <CopyCheck data-icon="inline-start" /> {t('rawLook.applyToAll')}
        </Button>
      }
    >
      <Field hint={t(value === 'camera' ? 'rawLook.cameraHint' : 'rawLook.neutralHint')}>
        <SegmentedToggle<RawLook>
          value={value}
          options={[
            { value: 'neutral', label: t('rawLook.neutral') },
            { value: 'camera', label: t('rawLook.camera') },
          ]}
          onChange={onChange}
          ariaLabel={t('rawLook.title')}
        />
      </Field>
    </SectionCard>
  )
}
