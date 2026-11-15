// SPDX-License-Identifier: GPL-3.0-only
// @shown both: in the gallery panels and in the preview (STYLE_RULE §4, where each card shows, checked by check-lego).
import { UserRound } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { AddAssist } from '@/components/forms/addassist'
import { SwitchField } from '@/components/forms/switch-field'
import { SectionCard } from '@/components/layout/section-card'
import type { RenderSettings } from '@/ipc/bindings'
import { emptyAddress } from '@/lib/address'
import { cardFor } from '@/lib/postcard-card'
import { patchPostcard } from './postcard-defaults'

type PostcardSenderCardProps = {
  settings: RenderSettings | null
  onChange: (change: (current: RenderSettings) => RenderSettings) => void
}

/**
 * The sender of Japanese and Korean cards (FR-136's first block, owner 2026-11-15): optional,
 * folded until its switch is on, one sender per country so switching cards never prints
 * another's. A European card has no sender line, so the card is not shown there.
 */
export function PostcardSenderCard({ settings, onChange }: PostcardSenderCardProps) {
  const { t, i18n } = useTranslation()
  const postcard = settings?.collage?.postcard
  const card = cardFor(i18n.language, postcard?.destination)
  if (!postcard?.enabled || card === 'europe') return null
  const country = card === 'japan' ? 'jp' : 'kr'
  return (
    <SectionCard icon={<UserRound />} title={t('postcard.sender')}>
      <SwitchField
        icon={<UserRound />}
        label={t('postcard.senderPrint')}
        checked={postcard.senderOn ?? false}
        onChange={(senderOn) => patchPostcard(onChange, { senderOn })}
        hint={t('postcard.senderHint')}
      >
        <AddAssist
          key={country}
          country={country}
          value={postcard.senders?.find((s) => s.country === country) ?? emptyAddress(country)}
          onCommit={(sender) =>
            patchPostcard(onChange, { senders: [...(postcard.senders ?? []).filter((s) => s.country !== country), sender] })
          }
          assisted={postcard.assisted ?? true}
          onAssisted={(assisted) => patchPostcard(onChange, { assisted })}
        />
      </SwitchField>
    </SectionCard>
  )
}
