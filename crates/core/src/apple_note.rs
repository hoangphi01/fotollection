// SPDX-License-Identifier: GPL-3.0-only
//! Apple's EXIF `MakerNote`, the few tags that sort a phone's roll (FR-141, ADR-0054): the Live
//! Photo's content identifier, the burst, the capture type and the photo's identifier. Its
//! presence alone says the phone's own camera took the shot, as apps that re-save a photo drop
//! it. Bounded (SECURITY.md §3): at most [`MAX_ENTRIES`] entries, strings of at most
//! [`MAX_STRING`] bytes, every offset checked against the note's length; anything else is
//! ignored, and a broken note reads as `None` or as the tags it still holds, never a panic.

/// The header Apple writes before its IFD.
const MAGIC: &[u8] = b"Apple iOS\0";
/// The IFD starts after the magic, a 2-byte version and the byte order `MM`.
const IFD_AT: usize = 14;
const MAX_ENTRIES: usize = 64;
const MAX_STRING: usize = 64;

const BURST_UUID: u16 = 0x000b;
const CONTENT_IDENTIFIER: u16 = 0x0011;
const IMAGE_CAPTURE_TYPE: u16 = 0x0014;
const LIVE_PHOTO_VIDEO_INDEX: u16 = 0x0017;
const PHOTO_IDENTIFIER: u16 = 0x002b;

/// What Apple's `MakerNote` says about a shot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppleNote {
    /// Shared with the Live Photo's video (its `QuickTime` `content.identifier`).
    pub content_id: Option<String>,
    /// Shared by every frame of one burst.
    pub burst_id: Option<String>,
    /// 1 `ProRAW`, 2 Portrait, 10 Photo… as the camera recorded it.
    pub capture_type: Option<i32>,
    /// Shared by the copies of one capture (HEIC and JPEG twins).
    pub photo_id: Option<String>,
    /// Set on a still that belongs to a Live Photo.
    pub live_photo_index: Option<u32>,
}

/// Reads the note from the bytes of EXIF's `MakerNote` field; `None` when it is not Apple's.
pub fn parse(note: &[u8]) -> Option<AppleNote> {
    if !note.starts_with(MAGIC) || note.get(12..14) != Some(b"MM".as_slice()) {
        return None;
    }
    let be16 = |at: usize| {
        note.get(at..at + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
    };
    let be32 = |at: usize| {
        note.get(at..at + 4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    };
    let count = usize::from(be16(IFD_AT)?).min(MAX_ENTRIES);
    let mut out = AppleNote::default();
    for i in 0..count {
        let at = IFD_AT + 2 + i * 12;
        let (Some(tag), Some(kind), Some(n), Some(value)) =
            (be16(at), be16(at + 2), be32(at + 4), be32(at + 8))
        else {
            break;
        };
        let n = usize::try_from(n).unwrap_or(usize::MAX);
        let string = || -> Option<String> {
            // ASCII (type 2): inline when 4 bytes or fewer, else at the offset.
            if kind != 2 || n == 0 || n > MAX_STRING {
                return None;
            }
            let bytes = if n <= 4 {
                note.get(at + 8..at + 8 + n)?
            } else {
                let from = usize::try_from(value).ok()?;
                note.get(from..from.checked_add(n)?)?
            };
            let text: String = bytes
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| char::from(b))
                .collect();
            let text = text.trim();
            (!text.is_empty() && text.chars().all(|c| c.is_ascii_graphic()))
                .then(|| text.to_string())
        };
        let integer = || -> Option<i64> {
            // SHORT (3), LONG (4), SSHORT (8), SLONG (9): one inline value.
            match kind {
                3 => be16(at + 8).map(i64::from),
                8 => be16(at + 8).map(|v| i64::from(v.cast_signed())),
                4 => Some(i64::from(value)),
                9 => Some(i64::from(value.cast_signed())),
                _ => None,
            }
        };
        match tag {
            CONTENT_IDENTIFIER => out.content_id = string(),
            BURST_UUID => out.burst_id = string(),
            PHOTO_IDENTIFIER => out.photo_id = string(),
            IMAGE_CAPTURE_TYPE => out.capture_type = integer().and_then(|v| i32::try_from(v).ok()),
            LIVE_PHOTO_VIDEO_INDEX => {
                out.live_photo_index = integer().and_then(|v| u32::try_from(v).ok());
            }
            _ => {}
        }
    }
    Some(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // a test's setup that fails is the test failing
mod tests {
    use super::*;

    /// An Apple `MakerNote` as the camera writes it: the header, then a big-endian IFD whose
    /// strings sit after it, their offsets counted from the note's start.
    fn note(entries: &[(u16, u16, &[u8])]) -> Vec<u8> {
        let mut head = MAGIC.to_vec();
        head.extend_from_slice(&[0, 1, b'M', b'M']);
        let count = u16::try_from(entries.len()).unwrap();
        head.extend_from_slice(&count.to_be_bytes());
        let mut data_at = IFD_AT + 2 + entries.len() * 12 + 4;
        let mut data = Vec::new();
        for &(tag, kind, value) in entries {
            head.extend_from_slice(&tag.to_be_bytes());
            head.extend_from_slice(&kind.to_be_bytes());
            let count = if kind == 2 { value.len() } else { 1 };
            head.extend_from_slice(&u32::try_from(count).unwrap().to_be_bytes());
            if kind == 2 && value.len() > 4 {
                head.extend_from_slice(&u32::try_from(data_at).unwrap().to_be_bytes());
                data.extend_from_slice(value);
                data_at += value.len();
            } else {
                let mut inline = [0u8; 4];
                inline[..value.len()].copy_from_slice(value);
                head.extend_from_slice(&inline);
            }
        }
        head.extend_from_slice(&[0, 0, 0, 0]);
        head.extend_from_slice(&data);
        head
    }

    const LIVE: &[u8] = b"5D4C1B2A-0F3E-4C59-9C2A-6E1B7D3F8A10\0";

    #[test]
    fn fr_141_apple_note_reads_the_live_photo_burst_and_capture_type() {
        let bytes = note(&[
            (BURST_UUID, 2, b"8F0A1C2B-0000-4000-8000-000000000001\0"),
            (CONTENT_IDENTIFIER, 2, LIVE),
            (IMAGE_CAPTURE_TYPE, 9, &10i32.to_be_bytes()),
            (LIVE_PHOTO_VIDEO_INDEX, 4, &3u32.to_be_bytes()),
            (
                PHOTO_IDENTIFIER,
                2,
                b"A1B2C3D4-1111-2222-3333-444455556666\0",
            ),
        ]);
        let n = parse(&bytes).unwrap();
        assert_eq!(
            n.content_id.as_deref(),
            Some("5D4C1B2A-0F3E-4C59-9C2A-6E1B7D3F8A10")
        );
        assert_eq!(
            n.burst_id.as_deref(),
            Some("8F0A1C2B-0000-4000-8000-000000000001")
        );
        assert_eq!(n.capture_type, Some(10));
        assert_eq!(n.live_photo_index, Some(3));
        assert_eq!(
            n.photo_id.as_deref(),
            Some("A1B2C3D4-1111-2222-3333-444455556666")
        );
    }

    #[test]
    fn fr_141_a_note_that_is_not_apples_reads_as_none() {
        assert_eq!(parse(b"Nikon\0\x02\x10\0\0MM\0\0"), None);
        assert_eq!(parse(&[]), None);
    }

    #[test]
    fn fr_141_a_broken_apple_note_never_panics() {
        let whole = note(&[
            (CONTENT_IDENTIFIER, 2, LIVE),
            (IMAGE_CAPTURE_TYPE, 9, &2i32.to_be_bytes()),
        ]);
        for cut in 0..whole.len() {
            let _ = parse(&whole[..cut]);
        }
        let mut bad = whole.clone();
        // An offset past the end, and a count claiming thousands of entries.
        bad[IFD_AT + 2 + 8..IFD_AT + 2 + 12].copy_from_slice(&u32::MAX.to_be_bytes());
        bad[IFD_AT..IFD_AT + 2].copy_from_slice(&u16::MAX.to_be_bytes());
        let n = parse(&bad).unwrap();
        assert_eq!(n.content_id, None);
        let mut seed = 0x9E37_79B9_u32;
        for _ in 0..2000 {
            let mut m = whole.clone();
            for _ in 0..4 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let i = usize::try_from(seed).unwrap() % m.len();
                m[i] = m[i].wrapping_add(u8::try_from(seed >> 24).unwrap() | 1);
            }
            let _ = parse(&m);
        }
    }
}
