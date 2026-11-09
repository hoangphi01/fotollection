// SPDX-License-Identifier: GPL-3.0-only
//! The ZIP container both of the app's bundles share (ADR-0016, FR-082): a manifest listing
//! every other part with its size and BLAKE3, parts named from each format's allowlist, sizes
//! and the compression ratio capped before anything is decompressed, each part read up to its
//! own limit whatever its header claims. `bundle.rs` (`.fotollection`) and `settings_bundle.rs`
//! (`.fotosettings`) differ only in their allowlist, their format name and their parts' readers.

use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Read, Write};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const MANIFEST: &str = "manifest.json";
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
pub const ZIP_MAGIC: &[u8] = b"PK\x03\x04";

/// One part listed in `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartEntry {
    pub path: String,
    pub bytes: u64,
    /// Lowercase hex BLAKE3 of the part.
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub schema_version: u32,
    pub app_version: String,
    pub parts: Vec<PartEntry>,
}

/// A format's allowlist: the kind of a part from its name (`None`: refused) and its limit.
pub trait PartKind: Copy {
    fn parse(name: &str) -> Option<Self>;
    fn limit(self) -> u64;
}

/// Caps checked on the central directory, before decompression.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub entries: usize,
    pub total: u64,
    pub ratio: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ZipError {
    /// Not this format (another manifest `format`).
    WrongFormat,
    /// A newer `schema_version` than this app reads.
    Newer(u32),
    TooLarge,
    Corrupt(String),
}

fn corrupt(why: impl Into<String>) -> ZipError {
    ZipError::Corrupt(why.into())
}

pub fn hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub type Parts<P> = BTreeMap<String, (P, Vec<u8>)>;

/// The parts of a ZIP, the manifest checked against them and removed: names and declared
/// sizes first (no decompression), then each part read up to its own limit.
pub fn read<P: PartKind>(
    bytes: &[u8],
    limits: Limits,
    format: &str,
    version: u32,
) -> Result<(Manifest, Parts<P>), ZipError> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|e| corrupt(e.to_string()))?;
    if zip.len() > limits.entries {
        return Err(corrupt("too many parts"));
    }
    let limit = |name: &str, part: Option<P>| {
        if name == MANIFEST {
            Some(MAX_MANIFEST_BYTES)
        } else {
            part.map(P::limit)
        }
    };
    // Pass 1: names and declared sizes only, before any decompression.
    let mut total = 0u64;
    let mut seen = HashSet::new();
    let mut kinds = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        let entry = zip.by_index_raw(i).map_err(|e| corrupt(e.to_string()))?;
        let name = entry
            .name()
            .map_err(|_| corrupt("a part name is not valid UTF-8"))?
            .into_owned();
        let part = P::parse(&name);
        let cap = limit(&name, part).ok_or_else(|| corrupt(format!("unexpected part {name:?}")))?;
        if entry.encrypted() || !seen.insert(name.clone()) {
            return Err(corrupt(format!("invalid part {name:?}")));
        }
        let (size, packed) = (entry.size(), entry.compressed_size());
        if size > cap {
            return Err(ZipError::TooLarge);
        }
        if size > packed.max(1).saturating_mul(limits.ratio) {
            return Err(corrupt(format!("{name:?} is compressed suspiciously well")));
        }
        total = total.saturating_add(size);
        if total > limits.total {
            return Err(ZipError::TooLarge);
        }
        kinds.push((i, name, part, cap));
    }
    // Pass 2: read each part, never more than its limit, whatever the header claims.
    let mut manifest = None;
    let mut data = Parts::new();
    for (i, name, part, cap) in kinds {
        let entry = zip.by_index(i).map_err(|e| corrupt(e.to_string()))?;
        let mut buf = Vec::new();
        entry
            .take(cap + 1)
            .read_to_end(&mut buf)
            .map_err(|e| corrupt(e.to_string()))?;
        if buf.len() as u64 > cap {
            return Err(ZipError::TooLarge);
        }
        match part {
            Some(part) if name != MANIFEST => {
                data.insert(name, (part, buf));
            }
            _ => manifest = Some(buf),
        }
    }
    let manifest = parse_manifest(
        &manifest.ok_or_else(|| corrupt("no manifest"))?,
        format,
        version,
    )?;
    verify(&manifest, &data)?;
    Ok((manifest, data))
}

/// The manifest lists exactly the other parts, each with the right size and BLAKE3 sum.
fn verify<P>(manifest: &Manifest, data: &Parts<P>) -> Result<(), ZipError> {
    let listed: HashSet<&str> = manifest.parts.iter().map(|p| p.path.as_str()).collect();
    if listed.len() != manifest.parts.len() || listed.len() != data.len() {
        return Err(corrupt("the manifest does not match the parts"));
    }
    for entry in &manifest.parts {
        let (_, bytes) = data
            .get(&entry.path)
            .ok_or_else(|| corrupt(format!("{:?} is listed but missing", entry.path)))?;
        if bytes.len() as u64 != entry.bytes || hash(bytes) != entry.blake3 {
            return Err(corrupt(format!("{:?} is damaged", entry.path)));
        }
    }
    Ok(())
}

fn parse_manifest(bytes: &[u8], format: &str, version: u32) -> Result<Manifest, ZipError> {
    #[derive(Deserialize)]
    struct Header {
        format: String,
        schema_version: u32,
    }
    let header: Header = serde_json::from_slice(bytes).map_err(|e| corrupt(e.to_string()))?;
    if header.format != format {
        return Err(ZipError::WrongFormat);
    }
    if header.schema_version > version {
        return Err(ZipError::Newer(header.schema_version));
    }
    serde_json::from_slice(bytes).map_err(|e| corrupt(e.to_string()))
}

/// Writes `parts` behind their manifest, in the given order with the fixed ZIP epoch, so the
/// same content always gives the same bytes; `deflate` says which parts are compressed (text),
/// the others are stored (fonts and PNGs are compressed already).
pub fn write(
    format: &str,
    version: u32,
    parts: &[(String, Vec<u8>)],
    deflate: impl Fn(&str) -> bool,
) -> Result<Vec<u8>, String> {
    let manifest = Manifest {
        format: format.into(),
        schema_version: version,
        app_version: env!("CARGO_PKG_VERSION").into(),
        parts: parts
            .iter()
            .map(|(path, bytes)| PartEntry {
                path: path.clone(),
                bytes: bytes.len() as u64,
                blake3: hash(bytes),
            })
            .collect(),
    };
    let manifest = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    zip.start_file(MANIFEST, deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(&manifest).map_err(|e| e.to_string())?;
    for (path, bytes) in parts {
        let options = if deflate(path) { deflated } else { stored };
        zip.start_file(path.as_str(), options)
            .map_err(|e| e.to_string())?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}
