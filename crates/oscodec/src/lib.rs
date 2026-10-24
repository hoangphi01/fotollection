// SPDX-License-Identifier: GPL-3.0-only
//! HEIC/HEIF decoding through the operating system's codecs (FR-001, ADR-0017): `ImageIO` on
//! macOS and WIC on Windows. This keeps the HEVC decoder and its licence with the OS, so
//! Fotollection links no LGPL library.
//!
//! This is the only crate that uses `unsafe` (FFI). Each block carries a `// SAFETY:`
//! comment, and the public API is safe (`docs/STYLE_RULE.md` §2). The size limits are checked
//! from the image header before any pixel is decoded (SECURITY.md §3).

#![allow(unsafe_code)]

/// A decoded image: 8-bit RGB, upright as stored (EXIF orientation is applied by the caller),
/// transparency flattened onto white, plus the embedded ICC profile when there is one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsImage {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
    pub icc: Option<Vec<u8>>,
}

/// Decompression-bomb limits, checked before decoding.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_edge: u32,
    pub max_pixels: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OsCodecError {
    /// This operating system has no HEIF codec we use.
    #[error("HEIC is not supported on this platform")]
    Unsupported,
    /// Windows without the "HEIF Image Extensions" / "HEVC Video Extensions".
    #[error("the HEIC codec is not installed")]
    CodecMissing,
    #[error("the image is too large ({0}×{1})")]
    TooLarge(u32, u32),
    #[error("not a decodable HEIC: {0}")]
    Decode(String),
}

fn check(width: usize, height: usize, limits: Limits) -> Result<(u32, u32), OsCodecError> {
    let (Ok(w), Ok(h)) = (u32::try_from(width), u32::try_from(height)) else {
        return Err(OsCodecError::TooLarge(u32::MAX, u32::MAX));
    };
    if w == 0 || h == 0 {
        return Err(OsCodecError::Decode("empty image".into()));
    }
    if w > limits.max_edge || h > limits.max_edge || u64::from(w) * u64::from(h) > limits.max_pixels
    {
        return Err(OsCodecError::TooLarge(w, h));
    }
    Ok((w, h))
}

/// True when HEIF files can be decoded on this machine right now.
pub fn heif_available() -> bool {
    imp::available()
}

/// Pixel size from the header, without decoding the image.
pub fn heif_dimensions(bytes: &[u8]) -> Result<(u32, u32), OsCodecError> {
    imp::dimensions(bytes)
}

/// Decodes the primary image of a HEIF file.
pub fn decode_heif(bytes: &[u8], limits: Limits) -> Result<OsImage, OsCodecError> {
    imp::decode(bytes, limits)
}

#[cfg(target_os = "macos")]
mod imp {
    use std::ffi::c_void;

    use objc2_core_foundation::{CFData, CFRetained, CGPoint, CGRect, CGSize};
    use objc2_core_graphics::{
        CGBitmapContextCreate, CGColorSpace, CGColorSpaceModel, CGContext, CGImage,
        CGImageAlphaInfo,
    };
    use objc2_image_io::CGImageSource;

    use super::{Limits, OsCodecError, OsImage, check};

    pub fn available() -> bool {
        true
    }

    fn primary_image(bytes: &[u8]) -> Result<CFRetained<CGImage>, OsCodecError> {
        let data = CFData::from_bytes(bytes);
        // SAFETY: `data` is a valid CFData and no options dictionary is passed.
        let source = unsafe { CGImageSource::with_data(&data, None) }
            .ok_or_else(|| OsCodecError::Decode("ImageIO cannot read it".into()))?;
        // SAFETY: `source` is a valid image source created above.
        let index = unsafe { source.primary_image_index() };
        // SAFETY: as above; `index` comes from the source itself, and no options are passed.
        // ImageIO decodes lazily, so this does not decode pixels yet.
        unsafe { source.image_at_index(index, None) }
            .ok_or_else(|| OsCodecError::Decode("no image in the file".into()))
    }

    pub fn dimensions(bytes: &[u8]) -> Result<(u32, u32), OsCodecError> {
        let image = primary_image(bytes)?;
        let (w, h) = (CGImage::width(Some(&image)), CGImage::height(Some(&image)));
        u32::try_from(w)
            .ok()
            .zip(u32::try_from(h).ok())
            .ok_or(OsCodecError::TooLarge(u32::MAX, u32::MAX))
    }

    pub fn decode(bytes: &[u8], limits: Limits) -> Result<OsImage, OsCodecError> {
        let image = primary_image(bytes)?;
        let (width, height) = (CGImage::width(Some(&image)), CGImage::height(Some(&image)));
        // Limits first: the bitmap below is the large allocation.
        let (w, h) = check(width, height, limits)?;

        // Draw in the image's own RGB colour space so the pixel values are unchanged and its
        // ICC profile stays valid for them; anything else (grey, CMYK) becomes device RGB.
        let source_space = CGImage::color_space(Some(&image));
        let icc = source_space
            .as_deref()
            .and_then(|s| CGColorSpace::icc_data(Some(s)))
            .map(|d| d.to_vec());
        let space = source_space
            .filter(|s| CGColorSpace::model(Some(s)) == CGColorSpaceModel::RGB)
            .or_else(CGColorSpace::new_device_rgb)
            .ok_or_else(|| OsCodecError::Decode("no RGB colour space".into()))?;
        let icc = if CGColorSpace::model(Some(&space)) == CGColorSpaceModel::RGB {
            icc
        } else {
            None
        };

        let row = width * 4;
        let mut rgbx = vec![0u8; row * height];
        // SAFETY: `rgbx` holds exactly `row * height` bytes with `row = 4 * width`, matching
        // 8 bits per component and RGBX (`NoneSkipLast`); it outlives `context`, which is
        // dropped below before the buffer is read.
        let context = unsafe {
            CGBitmapContextCreate(
                rgbx.as_mut_ptr().cast::<c_void>(),
                width,
                height,
                8,
                row,
                Some(&space),
                CGImageAlphaInfo::NoneSkipLast.0,
            )
        }
        .ok_or_else(|| OsCodecError::Decode("cannot allocate the bitmap".into()))?;
        #[allow(clippy::cast_precision_loss)] // within MAX_EDGE, exact in f64
        let rect = CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(width as f64, height as f64),
        );
        // Transparent areas print as white, like PNG input.
        CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, 1.0);
        CGContext::fill_rect(Some(&context), rect);
        CGContext::draw_image(Some(&context), rect, Some(&image));
        drop(context);

        let rgb = rgbx
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|&[r, g, b, _]| [r, g, b])
            .collect();
        Ok(OsImage {
            width: w,
            height: h,
            rgb,
            icc,
        })
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
    use windows::Win32::Graphics::Imaging::{
        CLSID_WICImagingFactory, GUID_ContainerFormatHeif, GUID_WICPixelFormat24bppRGB,
        IWICBitmapFrameDecode, IWICImagingFactory, WICBitmapDitherTypeNone,
        WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
    };

    use super::{Limits, OsCodecError, OsImage, check};

    /// `WINCODEC_ERR_COMPONENTNOTFOUND`: no decoder for the format (codec not installed).
    const COMPONENT_NOT_FOUND: i32 = 0x8898_2F50_u32.cast_signed();

    fn factory() -> Result<IWICImagingFactory, OsCodecError> {
        // SAFETY: initialises COM for this thread; repeated calls are allowed and a thread
        // already in another apartment returns RPC_E_CHANGED_MODE, which is fine for WIC.
        let init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if init.is_err() && init != RPC_E_CHANGED_MODE {
            return Err(OsCodecError::Decode(format!("COM: {init:?}")));
        }
        // SAFETY: standard creation of the WIC factory in-process.
        unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| OsCodecError::Decode(e.to_string()))
    }

    fn frame(bytes: &[u8]) -> Result<(IWICImagingFactory, IWICBitmapFrameDecode), OsCodecError> {
        let factory = factory()?;
        // SAFETY: the stream reads `bytes`, which outlives every use of the decoder here.
        let decoder = unsafe {
            let stream = factory
                .CreateStream()
                .map_err(|e| OsCodecError::Decode(e.to_string()))?;
            stream
                .InitializeFromMemory(bytes)
                .map_err(|e| OsCodecError::Decode(e.to_string()))?;
            factory.CreateDecoderFromStream(
                &stream,
                std::ptr::null(),
                WICDecodeMetadataCacheOnDemand,
            )
        }
        .map_err(|e| {
            if e.code().0 == COMPONENT_NOT_FOUND {
                OsCodecError::CodecMissing
            } else {
                OsCodecError::Decode(e.to_string())
            }
        })?;
        // SAFETY: frame 0 is the primary image; the decoder is valid.
        let frame =
            unsafe { decoder.GetFrame(0) }.map_err(|e| OsCodecError::Decode(e.to_string()))?;
        Ok((factory, frame))
    }

    pub fn available() -> bool {
        let Ok(factory) = factory() else {
            return false;
        };
        // SAFETY: asks WIC for a decoder of the HEIF container; nothing is decoded.
        unsafe { factory.CreateDecoder(&GUID_ContainerFormatHeif, std::ptr::null()) }.is_ok()
    }

    fn size(frame: &IWICBitmapFrameDecode) -> Result<(u32, u32), OsCodecError> {
        let (mut w, mut h) = (0u32, 0u32);
        // SAFETY: both out-pointers are valid locals.
        unsafe { frame.GetSize(&raw mut w, &raw mut h) }
            .map_err(|e| OsCodecError::Decode(e.to_string()))?;
        Ok((w, h))
    }

    pub fn dimensions(bytes: &[u8]) -> Result<(u32, u32), OsCodecError> {
        let (_, frame) = frame(bytes)?;
        size(&frame)
    }

    pub fn decode(bytes: &[u8], limits: Limits) -> Result<OsImage, OsCodecError> {
        let (factory, frame) = frame(bytes)?;
        let (w, h) = size(&frame)?;
        let (w, h) = check(w as usize, h as usize, limits)?;
        let stride = w * 3;
        let mut rgb = vec![0u8; stride as usize * h as usize];
        // SAFETY: the converter reads `frame`; `rgb` holds exactly `stride * h` bytes and
        // the whole image (null rect) is copied into it.
        unsafe {
            let converter = factory
                .CreateFormatConverter()
                .map_err(|e| OsCodecError::Decode(e.to_string()))?;
            converter
                .Initialize(
                    &frame,
                    &GUID_WICPixelFormat24bppRGB,
                    WICBitmapDitherTypeNone,
                    None,
                    0.0,
                    WICBitmapPaletteTypeCustom,
                )
                .map_err(|e| OsCodecError::Decode(e.to_string()))?;
            converter
                .CopyPixels(std::ptr::null(), stride, &mut rgb)
                .map_err(|e| OsCodecError::Decode(e.to_string()))?;
        }
        // The embedded ICC profile is not read on Windows yet; colours are left as stored.
        Ok(OsImage {
            width: w,
            height: h,
            rgb,
            icc: None,
        })
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod imp {
    use super::{Limits, OsCodecError, OsImage};

    pub fn available() -> bool {
        false
    }

    pub fn dimensions(_: &[u8]) -> Result<(u32, u32), OsCodecError> {
        Err(OsCodecError::Unsupported)
    }

    pub fn decode(_: &[u8], _: Limits) -> Result<OsImage, OsCodecError> {
        Err(OsCodecError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: Limits = Limits {
        max_edge: 30_000,
        max_pixels: 200_000_000,
    };

    #[test]
    fn fr_001_heic_garbage_is_an_error_not_a_panic() {
        let result = decode_heif(b"\0\0\0\x18ftypheic\0\0\0\0not really an image", LIMITS);
        assert!(result.is_err());
    }

    #[test]
    fn limits_are_checked_before_decoding() {
        let tight = Limits {
            max_edge: 10,
            max_pixels: 100,
        };
        assert_eq!(check(11, 5, tight), Err(OsCodecError::TooLarge(11, 5)));
        assert_eq!(check(10, 11, tight), Err(OsCodecError::TooLarge(10, 11)));
        assert_eq!(check(10, 10, tight), Ok((10, 10)));
        assert!(check(0, 10, tight).is_err());
    }

    /// A real HEIC made by macOS itself from a synthetic image (no photo is committed).
    #[cfg(target_os = "macos")]
    #[test]
    fn fr_001_heic_decodes_on_macos() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("fotollection-heic-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        // A 64×48 TIFF with a red left half and a blue right half, written by hand.
        let tiff = dir.join("in.tiff");
        std::fs::write(&tiff, tiny_tiff(64, 48))?;
        let heic = dir.join("out.heic");
        let status = std::process::Command::new("sips")
            .args(["-s", "format", "heic"])
            .arg(&tiff)
            .arg("--out")
            .arg(&heic)
            .output()?;
        assert!(status.status.success(), "sips failed: {status:?}");
        let bytes = std::fs::read(&heic)?;
        assert_eq!(heif_dimensions(&bytes)?, (64, 48));
        let image = decode_heif(&bytes, LIMITS)?;
        assert_eq!((image.width, image.height), (64, 48));
        assert_eq!(image.rgb.len(), 64 * 48 * 3);
        let left = &image.rgb[(24 * 64 + 8) * 3..][..3];
        let right = &image.rgb[(24 * 64 + 56) * 3..][..3];
        assert!(left[0] > 180 && left[2] < 90, "left is red: {left:?}");
        assert!(right[2] > 180 && right[0] < 90, "right is blue: {right:?}");
        assert!(image.icc.is_some(), "ImageIO reports the colour space");
        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    /// An uncompressed RGB baseline TIFF: left half red, right half blue.
    #[cfg(target_os = "macos")]
    fn tiny_tiff(w: u16, h: u16) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(usize::from(w) * usize::from(h) * 3);
        for _ in 0..h {
            for x in 0..w {
                pixels.extend_from_slice(if x < w / 2 {
                    &[220, 20, 20]
                } else {
                    &[20, 20, 220]
                });
            }
        }
        let entries: [(u16, u16, u32, u32); 9] = [
            (256, 3, 1, u32::from(w)),                             // width
            (257, 3, 1, u32::from(h)),                             // height
            (258, 3, 1, 8),            // bits per sample (same for all three)
            (259, 3, 1, 1),            // no compression
            (262, 3, 1, 2),            // RGB
            (273, 4, 1, 8),            // strip offset: pixels right after the header
            (277, 3, 1, 3),            // samples per pixel
            (278, 3, 1, u32::from(h)), // rows per strip
            (279, 4, 1, u32::try_from(pixels.len()).unwrap_or(0)), // strip bytes
        ];
        let ifd_offset = 8 + u32::try_from(pixels.len()).unwrap_or(0);
        let mut out = b"II*\0".to_vec();
        out.extend_from_slice(&ifd_offset.to_le_bytes());
        out.extend_from_slice(&pixels);
        out.extend_from_slice(&u16::try_from(entries.len()).unwrap_or(0).to_le_bytes());
        for (tag, kind, count, value) in entries {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&kind.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.extend_from_slice(&0u32.to_le_bytes());
        out
    }
}
