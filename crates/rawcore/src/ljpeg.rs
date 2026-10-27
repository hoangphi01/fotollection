// SPDX-License-Identifier: GPL-3.0-only
//! Lossless JPEG (ITU T.81, Annex H, the "process 14" used by DNG, CR2 and NEF): Huffman
//! coded differences from one of seven predictors. Written from the standard.

use crate::RawError;

/// The decoded samples: `height` rows of `width × components` values, interleaved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub components: usize,
    pub precision: u8,
    pub samples: Vec<u16>,
}

/// A frame bigger than this is refused before any allocation.
const MAX_SAMPLES: usize = 400_000_000;

/// Codes up to this many bits are decoded with one table lookup.
const FAST_BITS: u32 = 12;

#[derive(Debug, Clone)]
struct Huffman {
    /// For each code length 1–16: the largest code of that length, or -1.
    maxcode: [i32; 17],
    /// Index into `values` of the first code of each length, minus that code.
    offset: [i32; 17],
    values: Vec<u8>,
    /// For each `FAST_BITS`-bit prefix: (code length, symbol), length 0 when the code is longer.
    fast: Vec<(u8, u8)>,
}

impl Huffman {
    fn new(counts: &[u8; 16], values: Vec<u8>) -> Result<Self, RawError> {
        let total: usize = counts.iter().map(|&c| usize::from(c)).sum();
        if total != values.len() || total > 256 {
            return Err(RawError::Corrupt("bad Huffman table"));
        }
        let mut maxcode = [-1i32; 17];
        let mut offset = [0i32; 17];
        let (mut code, mut index) = (0i32, 0i32);
        for len in 1..=16 {
            let n = i32::from(counts[len - 1]);
            offset[len] = index - code;
            if n > 0 {
                code += n;
                index += n;
                maxcode[len] = code - 1;
            }
            code <<= 1;
        }
        // Fill the lookup table: every prefix that starts with a short code maps to it.
        let mut fast = vec![(0u8, 0u8); 1 << FAST_BITS];
        let (mut code, mut k) = (0u32, 0usize);
        for len in 1..=16u32 {
            for _ in 0..counts[len as usize - 1] {
                if len <= FAST_BITS {
                    let shift = FAST_BITS - len;
                    let first = (code << shift) as usize;
                    for slot in fast.iter_mut().skip(first).take(1 << shift) {
                        #[allow(clippy::cast_possible_truncation)]
                        {
                            *slot = (len as u8, values[k]);
                        }
                    }
                }
                code += 1;
                k += 1;
            }
            code <<= 1;
        }
        Ok(Self {
            maxcode,
            offset,
            values,
            fast,
        })
    }
}

/// Reads bits from the entropy-coded data, undoing `FF 00` stuffing and stopping at a
/// marker (after which it reads zeros, as the standard allows for padding).
struct Bits<'a> {
    data: &'a [u8],
    at: usize,
    acc: u64,
    count: u32,
    /// A marker was met; `at` points at its `FF`.
    marker: bool,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8], at: usize) -> Self {
        Self {
            data,
            at,
            acc: 0,
            count: 0,
            marker: false,
        }
    }

    fn fill(&mut self) {
        while self.count <= 56 {
            let mut byte = 0u8;
            if !self.marker && self.at < self.data.len() {
                byte = self.data[self.at];
                if byte == 0xFF {
                    if self.data.get(self.at + 1) == Some(&0x00) {
                        self.at += 2;
                    } else {
                        self.marker = true;
                        byte = 0;
                    }
                } else {
                    self.at += 1;
                }
            }
            self.acc |= u64::from(byte) << (56 - self.count);
            self.count += 8;
        }
    }

    fn bits(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        if self.count < n {
            self.fill();
        }
        #[allow(clippy::cast_possible_truncation)]
        let v = (self.acc >> (64 - n)) as u32;
        self.acc <<= n;
        self.count -= n;
        v
    }

    fn decode(&mut self, table: &Huffman) -> Result<u8, RawError> {
        if self.count < 16 {
            self.fill();
        }
        #[allow(clippy::cast_possible_truncation)]
        let prefix = (self.acc >> (64 - FAST_BITS)) as usize;
        let (len, symbol) = table.fast[prefix];
        if len > 0 {
            self.acc <<= len;
            self.count -= u32::from(len);
            return Ok(symbol);
        }
        let mut code = 0i32;
        for len in 1..=16 {
            #[allow(clippy::cast_possible_wrap)]
            let bit = self.bits(1) as i32;
            code = (code << 1) | bit;
            if code <= table.maxcode[len] {
                let i = usize::try_from(table.offset[len] + code)
                    .map_err(|_| RawError::Corrupt("bad Huffman code"))?;
                return table
                    .values
                    .get(i)
                    .copied()
                    .ok_or(RawError::Corrupt("bad Huffman code"));
            }
        }
        Err(RawError::Corrupt("bad Huffman code"))
    }

    /// Skips to the restart marker after a restart interval and resets the reader.
    fn restart(&mut self) -> Result<(), RawError> {
        self.acc = 0;
        self.count = 0;
        self.marker = false;
        while self.at + 1 < self.data.len() {
            if self.data[self.at] == 0xFF && (0xD0..=0xD7).contains(&self.data[self.at + 1]) {
                self.at += 2;
                return Ok(());
            }
            self.at += 1;
        }
        Err(RawError::Truncated)
    }
}

/// The difference coded as `ssss` extra bits (T.81 F.1.2.1, H.1.2.2: 16 means 32768).
fn difference(bits: &mut Bits<'_>, ssss: u8) -> i32 {
    match ssss {
        0 => 0,
        16 => 32768,
        n => {
            let n = u32::from(n);
            #[allow(clippy::cast_possible_wrap)]
            let v = bits.bits(n) as i32;
            if v < (1 << (n - 1)) {
                v - (1 << n) + 1
            } else {
                v
            }
        }
    }
}

fn be16(data: &[u8], at: usize) -> Result<usize, RawError> {
    let b = data.get(at..at + 2).ok_or(RawError::Truncated)?;
    Ok(usize::from(u16::from_be_bytes([b[0], b[1]])))
}

/// Decodes one lossless JPEG.
#[allow(clippy::too_many_lines)]
pub fn decode(data: &[u8]) -> Result<Frame, RawError> {
    if data.get(..2) != Some(&[0xFF, 0xD8]) {
        return Err(RawError::Corrupt("not a JPEG"));
    }
    let mut tables: [Option<Huffman>; 4] = [None, None, None, None];
    let (mut width, mut height, mut components, mut precision) = (0usize, 0usize, 0usize, 0u8);
    let mut restart_interval = 0usize;
    let mut at = 2;
    loop {
        // Skip fill bytes and find the next marker.
        while data.get(at) == Some(&0xFF) && data.get(at + 1) == Some(&0xFF) {
            at += 1;
        }
        if data.get(at) != Some(&0xFF) {
            return Err(RawError::Corrupt("expected a JPEG marker"));
        }
        let marker = *data.get(at + 1).ok_or(RawError::Truncated)?;
        let len = be16(data, at + 2)?;
        let body = data.get(at + 4..at + 2 + len).ok_or(RawError::Truncated)?;
        match marker {
            // SOF3: the lossless frame.
            0xC3 => {
                precision = *body.first().ok_or(RawError::Truncated)?;
                height = be16(body, 1)?;
                width = be16(body, 3)?;
                components = usize::from(*body.get(5).ok_or(RawError::Truncated)?);
                if !(2..=16).contains(&precision) || components == 0 || components > 4 {
                    return Err(RawError::Unsupported("lossless JPEG layout"));
                }
                let total = width
                    .checked_mul(height)
                    .and_then(|n| n.checked_mul(components))
                    .ok_or(RawError::TooLarge(u64::MAX))?;
                if total > MAX_SAMPLES || total == 0 {
                    return Err(RawError::TooLarge(total as u64));
                }
            }
            0xC0..=0xC2 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => {
                return Err(RawError::Unsupported("a lossy JPEG"));
            }
            // DHT
            0xC4 => {
                let mut p = 0;
                while p < body.len() {
                    let id = usize::from(body[p] & 0x0F);
                    let counts: [u8; 16] = body
                        .get(p + 1..p + 17)
                        .and_then(|c| c.try_into().ok())
                        .ok_or(RawError::Truncated)?;
                    let n: usize = counts.iter().map(|&c| usize::from(c)).sum();
                    let values = body.get(p + 17..p + 17 + n).ok_or(RawError::Truncated)?;
                    let slot = tables.get_mut(id).ok_or(RawError::Corrupt("table id"))?;
                    *slot = Some(Huffman::new(&counts, values.to_vec())?);
                    p += 17 + n;
                }
            }
            // DRI
            0xDD => restart_interval = be16(body, 0)?,
            // SOS: the scan, then the entropy-coded data.
            0xDA => {
                if components == 0 {
                    return Err(RawError::Corrupt("scan before frame"));
                }
                let ns = usize::from(*body.first().ok_or(RawError::Truncated)?);
                if ns != components {
                    return Err(RawError::Unsupported("a scan of some components only"));
                }
                let mut table_of = Vec::with_capacity(ns);
                for i in 0..ns {
                    let td = usize::from(*body.get(2 + 2 * i).ok_or(RawError::Truncated)? >> 4);
                    let table = tables
                        .get(td)
                        .and_then(Option::as_ref)
                        .ok_or(RawError::Corrupt("missing Huffman table"))?;
                    table_of.push(table.clone());
                }
                let predictor = *body.get(1 + 2 * ns).ok_or(RawError::Truncated)?;
                let point = u32::from(*body.get(3 + 2 * ns).ok_or(RawError::Truncated)? & 0x0F);
                if !(1..=7).contains(&predictor) || point >= u32::from(precision) {
                    return Err(RawError::Corrupt("bad predictor"));
                }
                let start = at + 2 + len;
                let samples = scan(
                    data,
                    start,
                    (width, height, components),
                    &table_of,
                    predictor,
                    (precision, point),
                    restart_interval,
                )?;
                return Ok(Frame {
                    width,
                    height,
                    components,
                    precision,
                    samples,
                });
            }
            0xD9 => return Err(RawError::Corrupt("no scan")),
            _ => {}
        }
        at += 2 + len;
    }
}

fn scan(
    data: &[u8],
    start: usize,
    (width, height, comps): (usize, usize, usize),
    tables: &[Huffman],
    predictor: u8,
    (precision, point): (u8, u32),
    restart_interval: usize,
) -> Result<Vec<u16>, RawError> {
    let row_len = width * comps;
    let mut out = vec![0u16; row_len * height];
    let mut bits = Bits::new(data, start);
    let initial = 1i32 << (u32::from(precision) - point - 1);
    let mask = (1i32 << u32::from(precision)) - 1;
    let mut since_restart = 0usize;
    let mut first_row_of_interval = true;
    for y in 0..height {
        if restart_interval > 0 && since_restart == restart_interval {
            bits.restart()?;
            since_restart = 0;
            first_row_of_interval = true;
        }
        for x in 0..width {
            for (c, table) in tables.iter().enumerate().take(comps) {
                let i = y * row_len + x * comps + c;
                let ssss = bits.decode(table)?;
                if ssss > 16 {
                    return Err(RawError::Corrupt("bad difference size"));
                }
                let diff = difference(&mut bits, ssss);
                let left = || i32::from(out[i - comps]);
                let up = || i32::from(out[i - row_len]);
                let pred = if x == 0 && (y == 0 || first_row_of_interval) {
                    initial
                } else if y == 0 || first_row_of_interval {
                    left()
                } else if x == 0 {
                    up()
                } else {
                    let (ra, rb, rc) = (left(), up(), i32::from(out[i - row_len - comps]));
                    match predictor {
                        1 => ra,
                        2 => rb,
                        3 => rc,
                        4 => ra + rb - rc,
                        5 => ra + ((rb - rc) >> 1),
                        6 => rb + ((ra - rc) >> 1),
                        _ => (ra + rb) >> 1,
                    }
                };
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                {
                    out[i] = ((pred + diff) & mask) as u16;
                }
            }
            since_restart += 1;
        }
        first_row_of_interval = false;
    }
    if point > 0 {
        for v in &mut out {
            *v <<= point;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-made 3×2, one-component, 8-bit lossless JPEG with predictor 1 and a Huffman
    /// table where SSSS 0 is `0`, 1 is `10` and 2 is `110`. Samples: 128 130 129 / 127 127 128.
    fn tiny() -> Vec<u8> {
        let mut j = vec![0xFF, 0xD8];
        // SOF3: P=8, Y=2, X=3, Nf=1, (C=1, H/V=0x11, Tq=0)
        j.extend_from_slice(&[0xFF, 0xC3, 0, 11, 8, 0, 2, 0, 3, 1, 1, 0x11, 0]);
        // DHT: table 0, counts: one code of length 1, one of 2, one of 3; values 0, 1, 2.
        let mut dht = vec![0x00, 1, 1, 1];
        dht.extend_from_slice(&[0; 13]);
        dht.extend_from_slice(&[0, 1, 2]);
        j.extend_from_slice(&[0xFF, 0xC4, 0, u8::try_from(dht.len() + 2).unwrap_or(0)]);
        j.extend(dht);
        // SOS: Ns=1, (Cs=1, Td/Ta=0x00), Ss=1 (predictor), Se=0, Ah/Al=0
        j.extend_from_slice(&[0xFF, 0xDA, 0, 8, 1, 1, 0x00, 1, 0, 0]);
        // Differences, row 0 from 128 then left: 0, +2, -1; row 1 (first sample from above,
        // then left): -1, 0, +1.
        //   0      -> "0"
        //   +2     -> "110" + "10"
        //   -1     -> "10"  + "0"
        //   -1     -> "10"  + "0"
        //   0      -> "0"
        //   +1     -> "10"  + "1"
        // bits: 0 11010 100 100 0 101 -> 0110 1010 0100 0101 (+ padding 1s)
        j.extend_from_slice(&[0b0110_1010, 0b0100_0101]);
        j.extend_from_slice(&[0xFF, 0xD9]);
        j
    }

    #[test]
    fn decodes_a_hand_made_frame() -> Result<(), RawError> {
        let f = decode(&tiny())?;
        assert_eq!((f.width, f.height, f.components), (3, 2, 1));
        assert_eq!(f.samples, vec![128, 130, 129, 127, 127, 128]);
        Ok(())
    }

    #[test]
    fn damaged_frames_are_errors_not_panics() {
        let j = tiny();
        for cut in 0..j.len() {
            let _ = decode(&j[..cut]);
        }
        for i in 0..j.len() {
            let mut bad = j.clone();
            bad[i] ^= 0x5A;
            let _ = decode(&bad);
        }
    }
}
