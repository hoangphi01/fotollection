// SPDX-License-Identifier: GPL-3.0-only
//! Demosaicing with the gradient-corrected linear interpolation of Malvar, He and Cutler,
//! "High-quality linear interpolation for demosaicing of Bayer-patterned color images"
//! (ICASSP 2004): each missing colour is a bilinear estimate corrected by the Laplacian of
//! the colour known at that site, through fixed 5×5 kernels. Written from the paper.

use rayon::prelude::*;

use crate::cfa::{Mosaic, Site};

/// White-balanced, demosaiced camera RGB, linear, row by row, for the default crop.
#[derive(Debug, Clone)]
pub struct CameraRgb {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<f32>,
}

/// The balanced sensor values over the active area, black at 0, with a border of `pad`
/// pixels reflected around it so every method's reads are plain indexes. Reflection
/// (-v, 2(n-1)-v) keeps each site's parity, so a border pixel has the colour of the site it
/// copies; `pad` is even, so the pattern lines up with the padded grid too.
#[derive(Debug, Clone)]
pub struct Plane {
    pub width: usize,
    pub height: usize,
    pub pad: usize,
    pub data: Vec<f32>,
    /// The 2×2 filter colours from the active area's corner (as `Mosaic::pattern`).
    pub pattern: [Site; 4],
    /// `[x, y, width, height]` of the default crop inside the active area.
    pub crop: [usize; 4],
}

impl Plane {
    /// The filter colour at padded position `(x, y)`.
    pub fn site(&self, x: usize, y: usize) -> Site {
        self.pattern[(y & 1) * 2 + (x & 1)]
    }

    /// The padded index of crop pixel `(x, y)`.
    pub fn index_of_crop(&self, x: usize, y: usize) -> usize {
        (self.crop[1] + y + self.pad) * self.width + self.crop[0] + x + self.pad
    }
}

/// The border every method can read past the active area's edge.
pub const PAD: usize = 8;

/// The active area of `mosaic`, black-subtracted and multiplied by `gains[colour]` (white
/// balance before interpolation keeps the colour differences small, which every method
/// assumes), with a reflected border of [`PAD`] pixels.
pub fn balance(mosaic: &Mosaic, gains: [f32; 3]) -> Plane {
    let [top, left, bottom, right] = mosaic.active;
    let (aw, ah) = (right - left, bottom - top);
    let (pw, ph) = (aw + 2 * PAD, ah + 2 * PAD);
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    let reflect = |v: isize, n: usize| -> usize {
        let n = n as isize;
        let v = if v < 0 {
            -v
        } else if v >= n {
            2 * (n - 1) - v
        } else {
            v
        };
        v.clamp(0, n - 1) as usize
    };
    let mut data = vec![0f32; pw * ph];
    data.par_chunks_mut(pw).enumerate().for_each(|(r, row)| {
        #[allow(clippy::cast_possible_wrap)]
        let y = top + reflect(r as isize - PAD as isize, ah);
        for (c, v) in row.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_wrap)]
            let x = left + reflect(c as isize - PAD as isize, aw);
            *v = (mosaic.level(x, y) * gains[usize::from(mosaic.site(x, y))]).max(0.0);
        }
    });
    Plane {
        width: pw,
        height: ph,
        pad: PAD,
        data,
        pattern: mosaic.pattern,
        crop: mosaic.crop,
    }
}

/// Demosaics the default crop of `mosaic` with Malvar–He–Cutler, multiplying each site by
/// `gains[colour]` first. Values are clipped at `clip` so blown highlights stay neutral.
pub fn demosaic(mosaic: &Mosaic, gains: [f32; 3], clip: f32) -> CameraRgb {
    malvar(&balance(mosaic, gains), clip)
}

/// Malvar–He–Cutler on a balanced [`Plane`], for its default crop.
pub fn malvar(plane: &Plane, clip: f32) -> CameraRgb {
    let [_, _, w, h] = plane.crop;
    let pw = plane.width;
    let bal = &plane.data;
    let mut rgb = vec![0f32; w * h * 3];
    rgb.par_chunks_mut(w * 3).enumerate().for_each(|(r, row)| {
        for c in 0..w {
            let base = plane.index_of_crop(c, r);
            let (px, py) = (base % pw, base / pw);
            let site = plane.site(px, py);
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let p = |dx: isize, dy: isize| bal[(base as isize + dy * pw as isize + dx) as usize];
            let centre = p(0, 0);
            // Green at red or blue sites.
            let g_at_rb = (4.0 * centre + 2.0 * (p(0, -1) + p(0, 1) + p(-1, 0) + p(1, 0))
                - (p(0, -2) + p(0, 2) + p(-2, 0) + p(2, 0)))
                / 8.0;
            // Red or blue at green sites, from the row's or the column's neighbours.
            let along_row = (5.0 * centre + 4.0 * (p(-1, 0) + p(1, 0))
                - (p(-2, 0) + p(2, 0) + p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1))
                + f32::midpoint(p(0, -2), p(0, 2)))
                / 8.0;
            let along_col = (5.0 * centre + 4.0 * (p(0, -1) + p(0, 1))
                - (p(0, -2) + p(0, 2) + p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1))
                + f32::midpoint(p(-2, 0), p(2, 0)))
                / 8.0;
            // Red at blue or blue at red, from the diagonal neighbours.
            let opposite = (6.0 * centre + 2.0 * (p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1))
                - 1.5 * (p(0, -2) + p(0, 2) + p(-2, 0) + p(2, 0)))
                / 8.0;
            let (red, green, blue) = match site {
                0 => (centre, g_at_rb, opposite),
                2 => (opposite, g_at_rb, centre),
                _ => {
                    // Green on a red row has red to its left and right.
                    if plane.site(px + 1, py) == 0 {
                        (along_row, centre, along_col)
                    } else {
                        (along_col, centre, along_row)
                    }
                }
            };
            let o = &mut row[c * 3..c * 3 + 3];
            o[0] = red.clamp(0.0, clip);
            o[1] = green.clamp(0.0, clip);
            o[2] = blue.clamp(0.0, clip);
        }
    });
    CameraRgb {
        width: w,
        height: h,
        rgb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat grey behind an RGGB filter demosaics to the same grey everywhere.
    #[test]
    fn a_flat_field_stays_flat() -> Result<(), crate::RawError> {
        let (w, h) = (16, 12);
        let mosaic = Mosaic::new(
            w,
            h,
            vec![600; w * h],
            [0, 1, 1, 2],
            [100.0; 4],
            1100.0,
            [0, 0, h, w],
            [2, 2, 12, 8],
        )?;
        let out = demosaic(&mosaic, [1.0, 1.0, 1.0], 1.0);
        assert_eq!((out.width, out.height), (12, 8));
        assert!(out.rgb.iter().all(|v| (v - 0.5).abs() < 1e-5));
        Ok(())
    }

    /// A pure red scene (only red sites lit) comes out red, without green or blue.
    #[test]
    fn a_red_field_is_red() -> Result<(), crate::RawError> {
        let (w, h) = (16, 12);
        let mut data = vec![0u16; w * h];
        for y in 0..h {
            for x in 0..w {
                if y % 2 == 0 && x % 2 == 0 {
                    data[y * w + x] = 1000;
                }
            }
        }
        let mosaic = Mosaic::new(
            w,
            h,
            data,
            [0, 1, 1, 2],
            [0.0; 4],
            1000.0,
            [0, 0, h, w],
            [2, 2, 12, 8],
        )?;
        let out = demosaic(&mosaic, [1.0, 1.0, 1.0], 1.0);
        for px in out.rgb.as_chunks::<3>().0 {
            assert!(px[0] > 0.9 && px[1] < 0.1 && px[2] < 0.1, "{px:?}");
        }
        Ok(())
    }

    /// The pattern counts from the active area: with it at an odd row and column, the red
    /// sites are the odd ones of the stored image, and the scene still comes out red.
    #[test]
    fn a_red_field_is_red_behind_an_odd_active_area() -> Result<(), crate::RawError> {
        let (w, h) = (17, 13);
        let mut data = vec![0u16; w * h];
        for y in (1..h).step_by(2) {
            for x in (1..w).step_by(2) {
                data[y * w + x] = 1000;
            }
        }
        let mosaic = Mosaic::new(
            w,
            h,
            data,
            [0, 1, 1, 2],
            [0.0; 4],
            1000.0,
            [1, 1, h, w],
            [2, 2, 12, 8],
        )?;
        let out = demosaic(&mosaic, [1.0, 1.0, 1.0], 1.0);
        for px in out.rgb.as_chunks::<3>().0 {
            assert!(px[0] > 0.9 && px[1] < 0.1 && px[2] < 0.1, "{px:?}");
        }
        Ok(())
    }
}
