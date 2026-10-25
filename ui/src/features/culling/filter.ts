// SPDX-License-Identifier: GPL-3.0-only
import type { CullFilter } from '@/ipc/bindings'

/** The default library filter (FR-022): every photo. */
export const ALL_PHOTOS: CullFilter = { flag: 'all', minRating: 0, folders: [] }
