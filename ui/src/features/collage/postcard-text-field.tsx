// SPDX-License-Identifier: GPL-3.0-only
import { Bold, Italic, Underline } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CommitInput } from '@/components/forms/commit-input'
import { Label } from '@/components/ui/label'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { commands, type PhotoId, type PostcardPart, type TextStyle } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

/** The limits Rust checks for each part (`PostcardPart::limits`). */
const LIMITS: Record<PostcardPart, { chars: number; lines: number }> = {
  message: { chars: 400, lines: 8 },
  address: { chars: 200, lines: 5 },
}

const STYLES = [
  { key: 'bold', icon: Bold },
  { key: 'italic', icon: Italic },
  { key: 'underline', icon: Underline },
] as const

const load = async (subject: string) => {
  const result = await commands.getPostcardText(Number(subject))
  return result.status === 'ok' ? result.data : null
}

type PostcardTextFieldProps = {
  photo: PhotoId
  part: PostcardPart
  revision: number
  /** How this part is set on every card, and the change of it. */
  style: TextStyle | undefined
  onStyle: (style: TextStyle) => void
}

/**
 * One part of a postcard's text (FR-073), the message or the address, typed for the chosen
 * card, with its bold, italic and underline for every card. It goes onto its lines as the
 * user pauses typing; what does not fit them is cut on the print, as the card shows.
 */
export function PostcardTextField({ photo, part, revision, style, onStyle }: PostcardTextFieldProps) {
  const { t } = useTranslation()
  const { chars, lines } = LIMITS[part]
  // The last answer holds its place while the next loads (LESSONS).
  const text = useLiveQuery(String(photo), revision, load).data?.[part] ?? ''
  const on = STYLES.filter((s) => style?.[s.key]).map((s) => s.key)
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between gap-2">
        <Label className="text-xs font-normal text-muted-foreground">{t(`postcard.${part}`)}</Label>
        <span className="ml-auto text-xs text-muted-foreground tabular-nums">
          {t('postcard.chars', { count: text.length, max: chars })}
        </span>
        <ToggleGroup
          type="multiple"
          variant="outline"
          size="sm"
          value={on}
          onValueChange={(keys) =>
            onStyle({ bold: keys.includes('bold'), italic: keys.includes('italic'), underline: keys.includes('underline') })
          }
          aria-label={t('postcard.style')}
        >
          {STYLES.map(({ key, icon: Icon }) => (
            <ToggleGroupItem key={key} value={key} className="size-7 px-0" aria-label={t(`postcard.styles.${key}`)} data-hint={t(`postcard.styles.${key}`)}>
              <Icon />
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>
      <CommitInput
        multiline
        value={text}
        placeholder={t(`postcard.${part}Placeholder`)}
        ariaLabel={t(`postcard.${part}`)}
        maxLength={chars}
        limit={(v) => v.split('\n').slice(0, lines).join('\n')}
        liveMs={600}
        onCommit={(next) => void commands.setPostcardText(photo, part, next.trim() === '' ? null : next)}
      />
    </div>
  )
}
