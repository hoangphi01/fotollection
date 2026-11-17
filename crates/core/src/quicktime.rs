// SPDX-License-Identifier: GPL-3.0-only
//! A video's metadata, for sorting a phone's roll (FR-141, ADR-0054): the Live Photo's content
//! identifier (shared with its still), when it was made and on what. The file's top-level
//! boxes are walked by their headers only, seeking past `mdat`, so a 200 MB video costs a few
//! reads; only `moov` is read, at most [`MAX_MOOV`] bytes. Bounded (SECURITY.md §3): sizes and
//! counts are checked, nesting is at most three levels, strings at most [`MAX_STRING`] bytes;
//! anything broken reads as the fields found so far, never a panic. Videos are never written.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use chrono::{DateTime, FixedOffset, NaiveDateTime};

const MAX_MOOV: u64 = 2 * 1024 * 1024;
const MAX_TOP_BOXES: usize = 64;
const MAX_KEYS: usize = 256;
const MAX_STRING: usize = 128;
/// Seconds from `QuickTime`'s epoch (1904-01-01) to the Unix epoch.
const EPOCH_1904: i64 = 2_082_844_800;

const CONTENT_ID_KEY: &str = "com.apple.quicktime.content.identifier";
const CREATION_KEY: &str = "com.apple.quicktime.creationdate";
const MAKE_KEY: &str = "com.apple.quicktime.make";
const MODEL_KEY: &str = "com.apple.quicktime.model";
const SOFTWARE_KEY: &str = "com.apple.quicktime.software";

/// What a video's `moov` says.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VideoMeta {
    /// Apple's `content.identifier`: a Live Photo's video carries its still's.
    pub content_id: Option<String>,
    /// Local wall-clock time with its offset, from Apple's `creationdate`.
    pub created_local: Option<DateTime<FixedOffset>>,
    /// UTC time from `mvhd`, which many tools fill with the copy's time or zero.
    pub created_utc: Option<NaiveDateTime>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub software: Option<String>,
    pub duration_s: Option<f32>,
}

/// Reads a MOV or MP4's metadata; `None` when it is not one or has no `moov`.
pub fn read_path(path: &Path) -> Option<VideoMeta> {
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    let mut at = 0u64;
    for _ in 0..MAX_TOP_BOXES {
        if at.checked_add(8)? > len {
            return None;
        }
        file.seek(SeekFrom::Start(at)).ok()?;
        let mut head = [0u8; 16];
        file.read_exact(&mut head[..8]).ok()?;
        let size32 = u32::from_be_bytes([head[0], head[1], head[2], head[3]]);
        let kind = [head[4], head[5], head[6], head[7]];
        let (size, header) = match size32 {
            0 => (len - at, 8),
            1 => {
                file.read_exact(&mut head[8..16]).ok()?;
                (u64::from_be_bytes(head[8..16].try_into().ok()?), 16)
            }
            n => (u64::from(n), 8),
        };
        if size < header || at.checked_add(size)? > len {
            return None;
        }
        if &kind == b"moov" {
            let body = size - header;
            if body > MAX_MOOV {
                return None;
            }
            let mut moov = vec![0u8; usize::try_from(body).ok()?];
            file.read_exact(&mut moov).ok()?;
            return Some(parse_moov(&moov));
        }
        at += size;
    }
    None
}

/// Parses a `moov` box's body (its header removed).
pub fn parse_moov(moov: &[u8]) -> VideoMeta {
    let mut out = VideoMeta::default();
    for (kind, body) in boxes(moov) {
        match &kind {
            b"mvhd" => mvhd(body, &mut out),
            b"meta" => meta(body, &mut out),
            b"udta" => {
                for (k, b) in boxes(body) {
                    if &k == b"meta" {
                        meta(b, &mut out);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// The child boxes of a body, as `(type, body)`; stops at the first broken header.
fn boxes(body: &[u8]) -> impl Iterator<Item = ([u8; 4], &[u8])> {
    let mut at = 0usize;
    std::iter::from_fn(move || {
        let head = body.get(at..at + 8)?;
        let size =
            usize::try_from(u32::from_be_bytes([head[0], head[1], head[2], head[3]])).ok()?;
        let kind = [head[4], head[5], head[6], head[7]];
        let end = if size == 0 {
            body.len()
        } else {
            at.checked_add(size)?
        };
        if size != 0 && size < 8 {
            return None;
        }
        let child = body.get(at + 8..end)?;
        at = end;
        Some((kind, child))
    })
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4)
        .map(|v| u32::from_be_bytes([v[0], v[1], v[2], v[3]]))
}

fn be64(b: &[u8], at: usize) -> Option<u64> {
    b.get(at..at + 8)
        .and_then(|v| v.try_into().ok())
        .map(u64::from_be_bytes)
}

#[allow(clippy::cast_precision_loss)]
fn mvhd(b: &[u8], out: &mut VideoMeta) {
    let version = b.first().copied();
    let (created, scale, duration) = match version {
        Some(0) => (
            be32(b, 4).map(u64::from),
            be32(b, 12),
            be32(b, 16).map(u64::from),
        ),
        Some(1) => (be64(b, 4), be32(b, 20), be64(b, 24)),
        _ => return,
    };
    if let Some(secs) = created
        .filter(|&s| s > 0)
        .and_then(|s| i64::try_from(s).ok())
    {
        out.created_utc = DateTime::from_timestamp(secs - EPOCH_1904, 0).map(|d| d.naive_utc());
    }
    if let (Some(scale), Some(duration)) = (scale.filter(|&s| s > 0), duration) {
        let s = duration as f64 / f64::from(scale);
        if s.is_finite() && (0.0..=86_400.0).contains(&s) {
            #[allow(clippy::cast_possible_truncation)]
            {
                out.duration_s = Some(s as f32);
            }
        }
    }
}

/// Apple's metadata: `keys` names the items, `ilst` holds them by 1-based index.
fn meta(b: &[u8], out: &mut VideoMeta) {
    // A QuickTime `meta` has no version field; an ISO one does. Try both.
    let body = if b.get(4..8) == Some(b"hdlr".as_slice()) {
        b
    } else {
        b.get(4..).unwrap_or_default()
    };
    let mut keys: Vec<String> = Vec::new();
    for (kind, child) in boxes(body) {
        match &kind {
            b"keys" => {
                let count = be32(child, 4)
                    .map_or(0, |n| usize::try_from(n).unwrap_or(0))
                    .min(MAX_KEYS);
                let mut at = 8;
                for _ in 0..count {
                    let Some(size) = be32(child, at).and_then(|n| usize::try_from(n).ok()) else {
                        break;
                    };
                    if size < 8 {
                        break;
                    }
                    let Some(name) = child.get(at + 8..at + size) else {
                        break;
                    };
                    keys.push(String::from_utf8_lossy(name).into_owned());
                    at += size;
                }
            }
            b"ilst" => {
                for (index, item) in boxes(child) {
                    let i = usize::try_from(u32::from_be_bytes(index)).unwrap_or(0);
                    let Some(key) = i.checked_sub(1).and_then(|i| keys.get(i)) else {
                        continue;
                    };
                    let Some(value) = boxes(item)
                        .find(|(k, _)| k == b"data")
                        .and_then(|(_, d)| text(d))
                    else {
                        continue;
                    };
                    match key.as_str() {
                        CONTENT_ID_KEY => out.content_id = Some(value),
                        CREATION_KEY => {
                            out.created_local =
                                DateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S%z").ok();
                        }
                        MAKE_KEY => out.make = Some(value),
                        MODEL_KEY => out.model = Some(value),
                        SOFTWARE_KEY => out.software = Some(value),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

/// A `data` box's UTF-8 value (type 1), trimmed and bounded.
fn text(data: &[u8]) -> Option<String> {
    if be32(data, 0)? != 1 {
        return None;
    }
    let bytes = data.get(8..)?;
    let bytes = &bytes[..bytes.len().min(MAX_STRING)];
    let s = String::from_utf8_lossy(bytes)
        .trim_matches(char::from(0))
        .trim()
        .to_string();
    (!s.is_empty() && !s.chars().any(char::is_control)).then_some(s)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // a test's setup that fails is the test failing
mod tests {
    use super::*;

    fn bx(kind: [u8; 4], body: &[u8]) -> Vec<u8> {
        let mut v = u32::try_from(body.len() + 8)
            .unwrap()
            .to_be_bytes()
            .to_vec();
        v.extend_from_slice(&kind);
        v.extend_from_slice(body);
        v
    }

    /// A Live Photo's video as an iPhone writes it: `ftyp`, a large `mdat`, then `moov`
    /// with `mvhd` and Apple's `meta` (keys and items).
    fn video(content_id: &str) -> Vec<u8> {
        let mut mvhd = vec![0u8; 100];
        mvhd[4..8].copy_from_slice(&(3_752_000_000u32).to_be_bytes());
        mvhd[12..16].copy_from_slice(&600u32.to_be_bytes());
        mvhd[16..20].copy_from_slice(&1680u32.to_be_bytes());
        let names = [CONTENT_ID_KEY, CREATION_KEY, MAKE_KEY];
        let mut keys = vec![0, 0, 0, 0];
        keys.extend_from_slice(&u32::try_from(names.len()).unwrap().to_be_bytes());
        for n in names {
            let mut k = u32::try_from(n.len() + 8).unwrap().to_be_bytes().to_vec();
            k.extend_from_slice(b"mdta");
            k.extend_from_slice(n.as_bytes());
            keys.extend(k);
        }
        let values = [content_id, "2022-01-27T17:31:20+0700", "Apple"];
        let mut ilst = Vec::new();
        for (i, v) in values.iter().enumerate() {
            let mut data = 1u32.to_be_bytes().to_vec();
            data.extend_from_slice(&[0, 0, 0, 0]);
            data.extend_from_slice(v.as_bytes());
            ilst.extend(bx(
                u32::try_from(i + 1).unwrap().to_be_bytes(),
                &bx(*b"data", &data),
            ));
        }
        let mut hdlr = vec![0u8; 8];
        hdlr.extend_from_slice(b"mdta");
        hdlr.extend_from_slice(&[0u8; 13]);
        let meta = [
            bx(*b"hdlr", &hdlr),
            bx(*b"keys", &keys),
            bx(*b"ilst", &ilst),
        ]
        .concat();
        let moov = [bx(*b"mvhd", &mvhd), bx(*b"meta", &meta)].concat();
        [
            bx(*b"ftyp", b"qt  \0\0\0\0qt  "),
            bx(*b"mdat", &vec![0u8; 50_000]),
            bx(*b"moov", &moov),
        ]
        .concat()
    }

    #[test]
    fn fr_141_a_live_photo_video_gives_its_content_identifier_past_the_head() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("IMG_0596.MOV");
        std::fs::write(&path, video("5D4C1B2A-0F3E-4C59-9C2A-6E1B7D3F8A10")).unwrap();
        let v = read_path(&path).unwrap();
        assert_eq!(
            v.content_id.as_deref(),
            Some("5D4C1B2A-0F3E-4C59-9C2A-6E1B7D3F8A10")
        );
        assert_eq!(v.make.as_deref(), Some("Apple"));
        assert_eq!(
            v.created_local.unwrap().to_rfc3339(),
            "2022-01-27T17:31:20+07:00"
        );
        assert_eq!(v.duration_s, Some(2.8));
        assert!(v.created_utc.is_some());
    }

    #[test]
    fn fr_141_a_live_photos_video_is_found_beside_its_still() {
        let dir = tempfile::tempdir().unwrap();
        let still = dir.path().join("IMG_8166.HEIC");
        std::fs::write(&still, b"").unwrap();
        std::fs::write(dir.path().join("IMG_8166.MOV"), video("LIVE-1")).unwrap();
        let found = crate::sorting::stacks::live_video(&still, "LIVE-1");
        assert_eq!(found, Some(dir.path().join("IMG_8166.MOV")));
        assert_eq!(crate::sorting::stacks::live_video(&still, "OTHER"), None);
    }

    #[test]
    fn fr_141_a_broken_video_never_panics() {
        let whole = video("ID");
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.mov");
        for cut in (0..whole.len()).step_by(997).chain([whole.len() - 3, 7, 0]) {
            std::fs::write(&path, &whole[..cut]).unwrap();
            let _ = read_path(&path);
        }
        let moov = &whole[whole.windows(4).position(|w| w == b"moov").unwrap() + 4..];
        assert!(parse_moov(moov).content_id.is_some());
        let mut seed = 0x2545_F491_u32;
        for _ in 0..3000 {
            let mut m = moov.to_vec();
            for _ in 0..6 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let i = usize::try_from(seed).unwrap() % m.len();
                m[i] = u8::try_from(seed >> 24).unwrap();
            }
            let _ = parse_moov(&m);
        }
    }
}
