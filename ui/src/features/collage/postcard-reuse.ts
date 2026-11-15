// SPDX-License-Identifier: GPL-3.0-only

/** Puts a card's message or recipient on the other cards (FR-077, FR-133): with `overwrite`,
 *  theirs too. Resolves to how many changed, or to an error's words. */
export type ReuseApply = (overwrite: boolean) => Promise<number | string>
