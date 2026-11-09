// SPDX-License-Identifier: GPL-3.0-only
//! Thumbnails through the photo's colour (FR-114, ADR-0047): the grid, the strips and the
//! quick preview show each photo as its look makes it, as Lightroom's grid does. The 400 px T1
//! on disk stays plain (the look tiles and the collage sheets start from it); its colour is laid
//! on when it is served: decoded, the look's table applied on rayon, encoded again. The bytes
//! are kept in memory, at most [`CAPACITY_BYTES`], oldest first out, so a grid scrolled back
//! costs nothing; a look changed gives a new key and the old bytes age out.

use crate::lock::Mutex;
use std::collections::VecDeque;

use crate::codec::{CodecError, EncodeOptions, JpegCodec, decode_image};
use crate::library::PhotoId;
use crate::looks::LookSettings;

/// The bytes of coloured thumbnails kept, at most (about 1,000 of them).
pub const CAPACITY_BYTES: usize = 32 << 20;
/// The quality they are encoded at, as T1's (`thumbs::THUMB_QUALITY`).
const QUALITY: u8 = 82;

/// A photo, the colour laid on, and the plain thumbnail it was laid on.
type Key = (PhotoId, u32, blake3::Hash);

/// The coloured thumbnails kept (owner: the app; bound: [`CAPACITY_BYTES`]; emptied when the
/// project changes, when memory runs short and by *Clear cache*, as the previews are).
#[derive(Default)]
pub struct LookThumbs {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    items: VecDeque<(Key, std::sync::Arc<Vec<u8>>)>,
    bytes: usize,
}

impl LookThumbs {
    /// `plain` through `look`, from memory or made now. A look that changes nothing gives
    /// `plain` itself, unchanged.
    pub fn get(
        &self,
        id: PhotoId,
        plain: &[u8],
        look: &LookSettings,
        codec: &dyn JpegCodec,
    ) -> Result<std::sync::Arc<Vec<u8>>, CodecError> {
        let colour = look.colour_key();
        if colour == 0 {
            return Ok(std::sync::Arc::new(plain.to_vec()));
        }
        let key = (id, colour, blake3::hash(plain));
        if let Some(found) = self.find(&key) {
            return Ok(found);
        }
        // Made outside the lock: two requests for one thumbnail may both make it, never wait.
        let made = std::sync::Arc::new(tint_jpeg(plain, look, codec)?);
        self.keep(key, made.clone());
        Ok(made)
    }

    fn find(&self, key: &Key) -> Option<std::sync::Arc<Vec<u8>>> {
        let inner = self.inner.lock().ok()?;
        inner
            .items
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn keep(&self, key: Key, bytes: std::sync::Arc<Vec<u8>>) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        if inner.items.iter().any(|(k, _)| *k == key) {
            return;
        }
        inner.bytes += bytes.len();
        inner.items.push_back((key, bytes));
        while inner.bytes > CAPACITY_BYTES {
            let Some((_, old)) = inner.items.pop_front() else {
                break;
            };
            inner.bytes -= old.len();
        }
    }

    /// The bytes kept now, for `usage()`.
    pub fn bytes(&self) -> usize {
        self.inner.lock().map_or(0, |i| i.bytes)
    }

    /// Forgets every coloured thumbnail (a project closed, memory short, *Clear cache*).
    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.items.clear();
            inner.bytes = 0;
        }
    }
}

/// A JPEG through `look`'s colour (its dehaze and table: no grain or vignette on a thumbnail), encoded
/// again for the screen.
pub fn tint_jpeg(
    plain: &[u8],
    look: &LookSettings,
    codec: &dyn JpegCodec,
) -> Result<Vec<u8>, CodecError> {
    let mut image = decode_image(codec, plain)?.image;
    look.draw(&mut image);
    codec.encode(&image, EncodeOptions::screen(QUALITY), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::default_codec;
    use crate::image::RgbImage;
    use crate::look_library::built_in;

    fn plain_jpeg() -> Vec<u8> {
        let mut image = RgbImage::new(64, 48);
        for (i, px) in image.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
            let x = u8::try_from(i % 64 * 4).unwrap_or(255);
            px.copy_from_slice(&[x, 180, 255 - x]);
        }
        default_codec()
            .encode(&image, EncodeOptions::screen(QUALITY), None)
            .unwrap_or_default()
    }

    fn look(id: &str) -> LookSettings {
        built_in()
            .into_iter()
            .find(|e| e.id == id)
            .map(|e| e.look)
            .unwrap_or_default()
    }

    /// FR-114: a look that changes nothing serves the plain thumbnail as it is; one that
    /// does colours it (black and white: no colour left) and is served again from memory.
    #[test]
    fn fr_114_thumbnails_show_the_look() -> Result<(), CodecError> {
        let codec = default_codec();
        let plain = plain_jpeg();
        let cache = LookThumbs::default();
        let same = cache.get(
            PhotoId(1),
            &plain,
            &look("builtin:original"),
            codec.as_ref(),
        )?;
        assert_eq!(*same, plain);
        assert_eq!(cache.bytes(), 0);
        let mono = cache.get(
            PhotoId(1),
            &plain,
            &look("builtin:mono-soft"),
            codec.as_ref(),
        )?;
        assert_ne!(*mono, plain);
        let image = decode_image(codec.as_ref(), &mono)?.image;
        let spread = image
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| p.iter().max().unwrap_or(&0) - p.iter().min().unwrap_or(&0))
            .max()
            .unwrap_or(0);
        assert!(
            spread <= 6,
            "a black and white thumbnail keeps colour: {spread}"
        );
        let kept = cache.bytes();
        assert!(kept > 0);
        let again = cache.get(
            PhotoId(1),
            &plain,
            &look("builtin:mono-soft"),
            codec.as_ref(),
        )?;
        assert!(std::sync::Arc::ptr_eq(&mono, &again));
        assert_eq!(cache.bytes(), kept);
        cache.clear();
        assert_eq!(cache.bytes(), 0);
        Ok(())
    }

    /// FR-114: the colour key is 0 without a look, the same for one colour under two names,
    /// and changes with the colour.
    #[test]
    fn fr_114_the_colour_key_follows_the_colour() {
        assert_eq!(LookSettings::default().colour_key(), 0);
        let c200 = look("builtin:c200-natural");
        let mut renamed = c200.clone();
        renamed.name = Some("Mine".into());
        renamed.base = None;
        assert_ne!(c200.colour_key(), 0);
        assert_eq!(c200.colour_key(), renamed.colour_key());
        let mut stronger = c200.clone();
        stronger.amount = 60;
        assert_ne!(c200.colour_key(), stronger.colour_key());
    }

    /// FR-114: the memory kept never passes its bound.
    #[test]
    fn fr_114_the_cache_stays_within_its_bound() {
        let cache = LookThumbs::default();
        for id in 0..40 {
            cache.keep(
                (PhotoId(id), 1, blake3::hash(&[0])),
                std::sync::Arc::new(vec![0; 1 << 20]),
            );
        }
        assert!(cache.bytes() <= CAPACITY_BYTES);
    }
}
