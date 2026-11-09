// SPDX-License-Identifier: GPL-3.0-only

/**
 * A fault injected on purpose, to prove a part fails alone (ADR-0048, UI-STATE-010). Only in
 * a build made with `VITE_FAULTS=1` (`scripts/ui-check.sh`); a release build has no such
 * variable, so Vite drops the check entirely (SECURITY: releases carry no developer tools).
 * The harness names the part to break in `window.__fotollectionFault`.
 */
export function faultAt(part: string): void {
  if (import.meta.env.VITE_FAULTS === '1' && (globalThis as { __fotollectionFault?: string }).__fotollectionFault === part) {
    throw new Error(`injected fault: ${part}`)
  }
}
