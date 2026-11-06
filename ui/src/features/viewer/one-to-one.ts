// SPDX-License-Identifier: GPL-3.0-only
import type { CropGeometry } from '@/ipc/bindings'

/**
 * The zoom at which one pixel of the photo meets one pixel of the screen (FR-095): the cropped
 * area's width in the photo's pixels, over the shown photo's width in device pixels at fit.
 * The cropped area is the largest one at the print's ratio, made smaller by the crop's own zoom;
 * in the frame's Fit mode the whole photo shows. `null` until both are known.
 */
export function oneToOneScale(geometry: CropGeometry | null | undefined, shown: HTMLElement | null): number | null {
  const onScreen = (shown?.offsetWidth ?? 0) * (window.devicePixelRatio || 1)
  if (!geometry || onScreen <= 0 || geometry.width <= 0 || geometry.height <= 0) return null
  let width = geometry.width
  if (!geometry.cropDisabled && geometry.targetWidth > 0 && geometry.targetHeight > 0) {
    const ratio = geometry.targetWidth / geometry.targetHeight
    const largest = geometry.width / geometry.height > ratio ? geometry.height * ratio : geometry.width
    width = largest / Math.max(1, geometry.crop?.zoom ?? 1)
  }
  return width / onScreen
}
