// SPDX-License-Identifier: GPL-3.0-only
import { Eye } from 'lucide-react'
import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { ToolbarButton } from '@/components/forms/toolbar-button'

type HoldOriginalButtonProps = {
  /** What a hold shows: the chosen photo, or the whole sheet with none chosen. */
  hint: string
  disabled?: boolean
  /** The pointer is over it: start loading the original, so the press shows it at once. */
  onWarm: () => void
  onHold: (held: boolean) => void
}

/**
 * *Hold for the original* in the sheet preview's toolbar (FR-096, owner, 2026-11-08): held
 * with the pointer, or with Space or Enter, the sheet shows the chosen photo (or both) without
 * its look; let go and the colour comes back. A button, because a hold on the sheet itself
 * picked its photo. While held it is the theme's colour pressed in, as every button held.
 */
export function HoldOriginalButton({ hint, disabled = false, onWarm, onHold }: HoldOriginalButtonProps) {
  const { t } = useTranslation()
  const held = useRef(false)
  const set = (on: boolean) => {
    if (held.current === on) return
    held.current = on
    onHold(on)
  }
  // The window losing focus mid-hold lets go too.
  useEffect(() => {
    const blur = () => {
      if (held.current) {
        held.current = false
        onHold(false)
      }
    }
    window.addEventListener('blur', blur)
    return () => window.removeEventListener('blur', blur)
  }, [onHold])
  useEffect(() => {
    if (disabled) set(false)
  })
  return (
    <ToolbarButton
      icon={<Eye />}
      // *Before*, as the photo preview's button in the same place; the hint says to hold.
      label={t('look.before')}
      labelFrom="5xl"
      data-hint={hint}
      disabled={disabled}
      onPointerEnter={onWarm}
      onPointerDown={(e) => {
        if (e.button !== 0) return
        e.currentTarget.setPointerCapture(e.pointerId)
        set(true)
      }}
      onPointerUp={() => set(false)}
      onPointerCancel={() => set(false)}
      onLostPointerCapture={() => set(false)}
      onKeyDown={(e) => {
        if ((e.key === ' ' || e.key === 'Enter') && !e.repeat) {
          e.preventDefault()
          onWarm()
          set(true)
        }
      }}
      onKeyUp={(e) => {
        if (e.key === ' ' || e.key === 'Enter') set(false)
      }}
      onBlur={() => set(false)}
    />
  )
}
