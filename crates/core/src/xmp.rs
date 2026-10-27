// SPDX-License-Identifier: GPL-3.0-only
//! XMP sidecars (FR-058): the `IMG_0416.xmp` or `IMG_0416.RW2.xmp` that Lightroom, Bridge,
//! Capture One or darktable write next to a photo. Read only, on import, for the star rating
//! and the reject flag (`xmp:Rating`, XMP Specification Part 2). Sidecars are untrusted:
//! at most 1 MB, no DTD (so no entities), namespaces resolved, any parse error ignored.

use std::io::Read;
use std::path::{Path, PathBuf};

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, XmlVersion};

/// The XMP basic namespace, whatever prefix a file binds it to.
const XMP_NS: &str = "http://ns.adobe.com/xap/1.0/";
/// Largest sidecar read; Lightroom's are a few kB, with history tens of kB.
pub const MAX_BYTES: u64 = 1024 * 1024;

/// What a sidecar says about a photo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sidecar {
    /// 1–5 stars, `None` when unrated.
    pub stars: Option<u8>,
    /// `xmp:Rating = -1`: rejected.
    pub rejected: bool,
}

/// The sidecar next to `photo`, if there is one: `name.xmp`, then `name.ext.xmp`, in either
/// case.
pub fn find(photo: &Path) -> Option<PathBuf> {
    let mut candidates = vec![photo.with_extension("xmp"), photo.with_extension("XMP")];
    for ext in ["xmp", "XMP"] {
        let mut name = photo.as_os_str().to_owned();
        name.push(".");
        name.push(ext);
        candidates.push(PathBuf::from(name));
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Reads the sidecar of `photo`; `None` when there is none, or it is too large or unreadable.
pub fn read_for(photo: &Path) -> Option<Sidecar> {
    let path = find(photo)?;
    let file = std::fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    parse(&bytes)
}

fn rating(text: &str) -> Option<Sidecar> {
    let value: f32 = text.trim().parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    let value = value.round() as i32;
    Some(match value {
        -1 => Sidecar {
            stars: None,
            rejected: true,
        },
        1..=5 => Sidecar {
            stars: u8::try_from(value).ok(),
            rejected: false,
        },
        _ => Sidecar::default(),
    })
}

/// The rating in an XMP packet, as an attribute (`xmp:Rating="4"`) or an element
/// (`<xmp:Rating>4</xmp:Rating>`). `None` when the packet has none, or is not well formed.
pub fn parse(bytes: &[u8]) -> Option<Sidecar> {
    let mut reader = NsReader::from_reader(bytes);
    let mut in_rating = false;
    loop {
        match reader.read_resolved_event().ok()? {
            // No DTD, so no entity can be declared, let alone expanded.
            (_, Event::DocType(_) | Event::Eof) => return None,
            (ResolveResult::Bound(ns), Event::Start(e))
                if ns.as_ref() == XMP_NS && e.local_name().as_ref() == "Rating" =>
            {
                in_rating = true;
            }
            (_, Event::Text(t)) if in_rating => return rating(&t.xml10_content()),
            (_, Event::End(_)) => in_rating = false,
            (_, Event::Start(e) | Event::Empty(e)) => {
                for attr in e.attributes().flatten() {
                    let (ns, local) = reader.resolver().resolve_attribute(attr.key);
                    if matches!(ns, ResolveResult::Bound(n) if n.as_ref() == XMP_NS)
                        && local.as_ref() == "Rating"
                    {
                        let value = attr.normalized_value(XmlVersion::Implicit1_0).ok()?;
                        return rating(&value);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIGHTROOM: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
   xmp:Rating="4"
   crs:HasCrop="True">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

    const ELEMENT: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:b="http://ns.adobe.com/xap/1.0/"><b:Rating>-1</b:Rating></rdf:Description>
</rdf:RDF></x:xmpmeta>"#;

    #[test]
    fn fr_058_reads_stars_as_an_attribute_and_reject_as_an_element() {
        assert_eq!(
            parse(LIGHTROOM.as_bytes()),
            Some(Sidecar {
                stars: Some(4),
                rejected: false
            })
        );
        // Any prefix works: the namespace decides.
        assert_eq!(
            parse(ELEMENT.as_bytes()),
            Some(Sidecar {
                stars: None,
                rejected: true
            })
        );
    }

    #[test]
    fn fr_058_ignores_other_namespaces_bad_values_and_broken_files() {
        let other = LIGHTROOM.replace("http://ns.adobe.com/xap/1.0/", "http://example.com/");
        assert_eq!(parse(other.as_bytes()), None);
        assert_eq!(
            parse(LIGHTROOM.replace("\"4\"", "\"9\"").as_bytes()),
            Some(Sidecar::default())
        );
        assert_eq!(
            parse(LIGHTROOM.replace("\"4\"", "\"NaN\"").as_bytes()),
            None
        );
        assert_eq!(parse(b"<x:xmpmeta"), None);
        assert_eq!(parse(b""), None);
        let dtd = format!("<!DOCTYPE x [<!ENTITY e SYSTEM \"file:///etc/passwd\">]>{LIGHTROOM}");
        assert_eq!(parse(dtd.as_bytes()), None);
    }

    #[test]
    fn fr_058_finds_both_sidecar_names_and_skips_large_files() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let raw = dir.path().join("P1000123.RW2");
        std::fs::write(&raw, b"raw")?;
        assert_eq!(read_for(&raw), None);
        std::fs::write(dir.path().join("P1000123.RW2.xmp"), LIGHTROOM)?;
        assert_eq!(read_for(&raw).and_then(|s| s.stars), Some(4));
        let jpeg = dir.path().join("IMG_0416.JPG");
        std::fs::write(dir.path().join("IMG_0416.xmp"), ELEMENT)?;
        assert!(read_for(&jpeg).is_some_and(|s| s.rejected));
        let mut big = LIGHTROOM.to_owned();
        big.push_str(&" ".repeat(usize::try_from(MAX_BYTES).unwrap_or(0)));
        std::fs::write(dir.path().join("IMG_0416.xmp"), big)?;
        assert_eq!(read_for(&jpeg), None);
        Ok(())
    }
}
