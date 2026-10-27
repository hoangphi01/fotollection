// SPDX-License-Identifier: GPL-3.0-only
//! Demosaicing with the gradient-corrected linear interpolation of Malvar, He and Cutler,
//! "High-quality linear interpolation for demosaicing of Bayer-patterned color images"
//! (ICASSP 2004): each missing colour is a bilinear estimate corrected by the Laplacian of
//! the colour known at that site, through fixed 5×5 kernels. Written from the paper.

use rayon::prelude::*;

use crate::cfa::Mosaic;

/// White-balanced, demosaiced camera RGB, linear, row by row, for the default crop.
#[derive(Debug, Clone)]
pub struct CameraRgb {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<f32>,
}

/// Demosaics the default crop of `mosaic`, multiplying each site by `gains[colour]` first
/// (white balance before interpolation keeps the colour differences small, which the
/// kernels assume). Values are clipped at `clip` so blown highlights stay neutral.
pub fn demosaic(mosaic: &Mosaic, gains: [f32; 3], clip: f32) -> CameraRgb {
    const PAD: usize = 2;
    let (w, h) = mosaic.size();
    let (ox, oy) = mosaic.origin();
    let [top, left, bottom, right] = mosaic.active;
    // Balanced values over the active area with a 2-pixel border reflected around it, so
    // every kernel read is a plain index. Reflection (-v, 2(n-1)-v) keeps each site's parity,
    // so a border pixel has the colour of the site it copies.
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
    let mut bal = vec![0f32; pw * ph];
    bal.par_chunks_mut(pw).enumerate().for_each(|(r, row)| {
        #[allow(clippy::cast_possible_wrap)]
        let y = top + reflect(r as isize - PAD as isize, ah);
        for (c, v) in row.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_wrap)]
            let x = left + reflect(c as isize - PAD as isize, aw);
            *v = (mosaic.level(x, y) * gains[usize::from(mosaic.site(x, y))]).max(0.0);
        }
    });
    let mut rgb = vec![0f32; w * h * 3];
    rgb.par_chunks_mut(w * 3).enumerate().for_each(|(r, row)| {
        let y = oy - top + r;
        for c in 0..w {
            let x = ox - left + c;
            let site = mosaic.site(left + x, top + y);
            // (x, y) in the active area is (x + PAD, y + PAD) in the padded buffer.
            let base = (y + PAD) * pw + x + PAD;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let p = |dx: isize, dy: isize| bal[(base as isize + dy * pw as isize + dx) as usize];
            let centre = p(0, 0);
            // Green at a red or blue site.
            let g_at_rb = (4.0 * centre + 2.0 * (p(0, -1) + p(0, 1) + p(-1, 0) + p(1, 0))
                - (p(0, -2) + p(0, 2) + p(-2, 0) + p(2, 0)))
                / 8.0;
            // At a green site: the colour of the horizontal neighbours, and of the vertical ones.
            let along_row = (5.0 * centre + 4.0 * (p(-1, 0) + p(1, 0))
                - (p(-2, 0) + p(2, 0) + p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1))
                + f32::midpoint(p(0, -2), p(0, 2)))
                / 8.0;
            let along_col = (5.0 * centre + 4.0 * (p(0, -1) + p(0, 1))
                - (p(0, -2) + p(0, 2) + p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1))
                + f32::midpoint(p(-2, 0), p(2, 0)))
                / 8.0;
            // Red at a blue site, or blue at a red site.
            let opposite = (6.0 * centre + 2.0 * (p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1))
                - 1.5 * (p(0, -2) + p(0, 2) + p(-2, 0) + p(2, 0)))
                / 8.0;
            let (red, green, blue) = match site {
                0 => (centre, g_at_rb, opposite),
                2 => (opposite, g_at_rb, centre),
                _ => {
                    // Green: which colour shares the row?
                    let row_site = mosaic.site(left + x + 1, top + y);
                    if row_site == 0 {
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
}
