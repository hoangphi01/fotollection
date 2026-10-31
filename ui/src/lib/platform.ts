// SPDX-License-Identifier: GPL-3.0-only

/** macOS gets the native menu and ⌘; Windows gets Ctrl and the UI's own shortcuts. */
export const isMac = navigator.userAgent.includes('Mac')

/** The modifier key as the keyboard hints write it. */
export const MOD = isMac ? '⌘' : 'Ctrl'
