// SPDX-License-Identifier: GPL-3.0-only
//! A bounded TIFF reader (TIFF 6.0): the byte order, image file directories and their
//! entries. RAW formats from DNG to ARW and RW2 are TIFF underneath.

use std::collections::HashMap;

use crate::RawError;

/// No real directory has more entries; a bigger count is a damaged file.
const MAX_ENTRIES: usize = 1024;
/// At most this many values are read from one entry.
const MAX_VALUES: usize = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Little,
    Big,
}

/// Reads integers of the file's byte order from a slice, with bounds checks.
#[derive(Debug, Clone, Copy)]
pub struct Bytes<'a> {
    pub data: &'a [u8],
    pub order: Order,
}

impl<'a> Bytes<'a> {
    pub fn slice(&self, at: usize, len: usize) -> Result<&'a [u8], RawError> {
        let end = at.checked_add(len).ok_or(RawError::Truncated)?;
        self.data.get(at..end).ok_or(RawError::Truncated)
    }

    pub fn u16(&self, at: usize) -> Result<u16, RawError> {
        let b = self.slice(at, 2)?;
        Ok(match self.order {
            Order::Little => u16::from_le_bytes([b[0], b[1]]),
            Order::Big => u16::from_be_bytes([b[0], b[1]]),
        })
    }

    pub fn u32(&self, at: usize) -> Result<u32, RawError> {
        let b = self.slice(at, 4)?;
        let a = [b[0], b[1], b[2], b[3]];
        Ok(match self.order {
            Order::Little => u32::from_le_bytes(a),
            Order::Big => u32::from_be_bytes(a),
        })
    }
}

/// One directory entry: its type, count, and where its values are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub kind: u16,
    pub count: u32,
    /// Absolute offset of the values (inside the entry when they fit in 4 bytes).
    pub at: usize,
}

fn type_size(kind: u16) -> Option<usize> {
    Some(match kind {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 | 13 => 4,
        5 | 10 | 12 => 8,
        _ => return None,
    })
}

/// A directory: its entries by tag, and the offset of the next one.
#[derive(Debug, Clone, Default)]
pub struct Ifd {
    pub entries: HashMap<u16, Entry>,
    pub next: u32,
}

/// A TIFF file: its bytes and byte order.
#[derive(Debug, Clone, Copy)]
pub struct Tiff<'a> {
    pub bytes: Bytes<'a>,
}

impl<'a> Tiff<'a> {
    /// Checks the header (`II*\0` / `MM\0*`, or another magic number as RW2 and ORF use).
    pub fn new(data: &'a [u8]) -> Result<Self, RawError> {
        let order = match data.get(..2) {
            Some(b"II") => Order::Little,
            Some(b"MM") => Order::Big,
            _ => return Err(RawError::Unsupported("not a TIFF file")),
        };
        Ok(Self {
            bytes: Bytes { data, order },
        })
    }

    /// The offset of the first directory, from the header.
    pub fn first_ifd(&self) -> Result<u32, RawError> {
        self.bytes.u32(4)
    }

    pub fn ifd(&self, offset: u32) -> Result<Ifd, RawError> {
        let at = offset as usize;
        let n = usize::from(self.bytes.u16(at)?);
        if n > MAX_ENTRIES {
            return Err(RawError::Corrupt("too many directory entries"));
        }
        let mut entries = HashMap::with_capacity(n);
        for i in 0..n {
            let e = at + 2 + 12 * i;
            let tag = self.bytes.u16(e)?;
            let kind = self.bytes.u16(e + 2)?;
            let count = self.bytes.u32(e + 4)?;
            let Some(size) = type_size(kind) else {
                continue;
            };
            let total = size
                .checked_mul(count as usize)
                .ok_or(RawError::Corrupt("entry too large"))?;
            let values = if total <= 4 {
                e + 8
            } else {
                self.bytes.u32(e + 8)? as usize
            };
            // The values must lie inside the file.
            self.bytes.slice(values, total)?;
            entries.insert(
                tag,
                Entry {
                    kind,
                    count,
                    at: values,
                },
            );
        }
        let next = self.bytes.u32(at + 2 + 12 * n).unwrap_or(0);
        Ok(Ifd { entries, next })
    }

    /// Integer values of an entry (BYTE, SHORT, LONG and their signed kinds).
    pub fn uints(&self, entry: &Entry) -> Result<Vec<u32>, RawError> {
        let n = (entry.count as usize).min(MAX_VALUES);
        (0..n)
            .map(|i| match entry.kind {
                1 | 6 | 7 => Ok(u32::from(
                    *self.bytes.slice(entry.at + i, 1)?.first().unwrap_or(&0),
                )),
                3 | 8 => self.bytes.u16(entry.at + 2 * i).map(u32::from),
                4 | 9 | 13 => self.bytes.u32(entry.at + 4 * i),
                _ => Err(RawError::Corrupt("not an integer entry")),
            })
            .collect()
    }

    /// Numeric values of an entry as `f64` (integers, rationals and floats).
    #[allow(clippy::cast_possible_wrap)]
    pub fn reals(&self, entry: &Entry) -> Result<Vec<f64>, RawError> {
        let n = (entry.count as usize).min(MAX_VALUES);
        (0..n)
            .map(|i| {
                Ok(match entry.kind {
                    5 => {
                        let (a, b) = (
                            self.bytes.u32(entry.at + 8 * i)?,
                            self.bytes.u32(entry.at + 8 * i + 4)?,
                        );
                        if b == 0 {
                            0.0
                        } else {
                            f64::from(a) / f64::from(b)
                        }
                    }
                    10 => {
                        let (a, b) = (
                            self.bytes.u32(entry.at + 8 * i)? as i32,
                            self.bytes.u32(entry.at + 8 * i + 4)? as i32,
                        );
                        if b == 0 {
                            0.0
                        } else {
                            f64::from(a) / f64::from(b)
                        }
                    }
                    11 => f64::from(f32::from_bits(self.bytes.u32(entry.at + 4 * i)?)),
                    12 => {
                        let (a, b) = (
                            u64::from(self.bytes.u32(entry.at + 8 * i)?),
                            u64::from(self.bytes.u32(entry.at + 8 * i + 4)?),
                        );
                        let bits = match self.bytes.order {
                            Order::Little => a | (b << 32),
                            Order::Big => (a << 32) | b,
                        };
                        f64::from_bits(bits)
                    }
                    8 => f64::from(self.bytes.u16(entry.at + 2 * i)? as i16),
                    9 => f64::from(self.bytes.u32(entry.at + 4 * i)? as i32),
                    _ => f64::from(self.uints(entry)?.get(i).copied().unwrap_or(0)),
                })
            })
            .collect()
    }

    /// The bytes an entry points at (UNDEFINED or BYTE data such as a CFA pattern).
    pub fn raw(&self, entry: &Entry) -> Result<&'a [u8], RawError> {
        let size = type_size(entry.kind).unwrap_or(1);
        self.bytes.slice(entry.at, size * entry.count as usize)
    }
}

impl Ifd {
    pub fn get(&self, tag: u16) -> Option<&Entry> {
        self.entries.get(&tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A little-endian TIFF with one directory: width 4000 (SHORT), a rational 3/2, and
    /// four BYTEs.
    fn sample() -> Vec<u8> {
        let mut t = b"II*\0".to_vec();
        t.extend_from_slice(&8u32.to_le_bytes());
        t.extend_from_slice(&3u16.to_le_bytes());
        let entry = |tag: u16, kind: u16, count: u32, value: u32| {
            let mut e = tag.to_le_bytes().to_vec();
            e.extend_from_slice(&kind.to_le_bytes());
            e.extend_from_slice(&count.to_le_bytes());
            e.extend_from_slice(&value.to_le_bytes());
            e
        };
        t.extend(entry(256, 3, 1, 4000));
        t.extend(entry(282, 5, 1, 8 + 2 + 36 + 4));
        t.extend(entry(33422, 1, 4, u32::from_le_bytes([0, 1, 1, 2])));
        t.extend_from_slice(&0u32.to_le_bytes());
        t.extend_from_slice(&3u32.to_le_bytes());
        t.extend_from_slice(&2u32.to_le_bytes());
        t
    }

    #[test]
    fn reads_entries_of_each_kind() -> Result<(), RawError> {
        let data = sample();
        let tiff = Tiff::new(&data)?;
        let ifd = tiff.ifd(tiff.first_ifd()?)?;
        let width = ifd.get(256).ok_or(RawError::Truncated)?;
        assert_eq!(tiff.uints(width)?, vec![4000]);
        assert_eq!(
            tiff.reals(ifd.get(282).ok_or(RawError::Truncated)?)?,
            vec![1.5]
        );
        assert_eq!(
            tiff.raw(ifd.get(33422).ok_or(RawError::Truncated)?)?,
            &[0, 1, 1, 2]
        );
        Ok(())
    }

    #[test]
    fn a_cut_file_is_an_error_not_a_panic() {
        let data = sample();
        for cut in 0..data.len() {
            let part = &data[..cut];
            if let Ok(tiff) = Tiff::new(part) {
                let _ = tiff.first_ifd().and_then(|o| tiff.ifd(o));
            }
        }
    }
}
