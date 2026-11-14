// SPDX-License-Identifier: GPL-3.0-only
import type { Market, PostcardDestination } from '@/ipc/bindings'

/**
 * The card the interface's language makes, as Rust decides it for the print
 * (`PostcardSettings::destination_for`, FR-130, FR-132): a はがき in Japanese, a 우편엽서 in
 * Korean, the chosen card in English and Vietnamese, Europe's elsewhere. The panel uses it only
 * to show the card's own controls.
 */
export function cardFor(language: string, destination: PostcardDestination | undefined): PostcardDestination {
  if (language === 'ja') return 'japan'
  if (language === 'ko') return 'korea'
  if (language === 'en' || language === 'vi') return destination ?? 'europe'
  return 'europe'
}

/** The market whose print sizes come first: the card's, else the interface's language's. */
export function marketFor(language: string, card: PostcardDestination): Market {
  if (card !== 'europe') return card
  if (language === 'ja') return 'japan'
  if (language === 'ko') return 'korea'
  return 'europe'
}

/** Whether a language may choose where its card goes (owner, 2026-11-15: English and Vietnamese). */
export const choosesCard = (language: string) => language === 'en' || language === 'vi'
