// SPDX-License-Identifier: GPL-3.0-only
//! Sony bodies in "PC Remote" USB mode (FR-062, FR-063), written from our notes in
//! `docs/cameras/sony.md`, observed on an a6400 (firmware 2.0). Builds what to send and reads
//! what comes back; the transport does the I/O.
//!
//! The body keeps its shutter locked until [`HANDSHAKE`] has run. Settings are then read all
//! at once ([`op::ALL_SETTINGS`], [`Settings::parse`]), set with [`op::SET_VALUE`], or moved
//! like a button or dial with [`op::CONTROL`]. A shot is announced by [`ev::NEW_PHOTO`] and
//! read from [`handle::PHOTO`]; live view is read from [`handle::LIVE_VIEW`] in a loop.

use crate::ptp::{MAX_LIST, PtpError, Reader, payload};

/// Sony's operation codes (vendor extension `0x11`).
pub mod op {
    /// Handshake step; parameter 1, 2 or 3 (see [`super::HANDSHAKE`]).
    pub const HANDSHAKE: u16 = 0x9201;
    /// The list of property codes the body offers; parameter `0xC8`.
    pub const PROPERTY_CODES: u16 = 0x9202;
    /// Sets a property to a value (data phase: the value).
    pub const SET_VALUE: u16 = 0x9205;
    /// Presses a button or moves a dial by steps (data phase: the value).
    pub const CONTROL: u16 = 0x9207;
    /// Every property's description and current value in one block.
    pub const ALL_SETTINGS: u16 = 0x9209;
}

/// Sony's event codes.
pub mod ev {
    /// A new photo can be read from [`super::handle::PHOTO`].
    pub const NEW_PHOTO: u16 = 0xC201;
    /// Follows [`NEW_PHOTO`] about 0.3–1 s later, same handle; meaning to confirm.
    pub const NEW_PHOTO_FOLLOW_UP: u16 = 0xC202;
    /// A property changed; parameter = its code.
    pub const PROPERTY_CHANGED: u16 = 0xC203;
}

/// Fixed object handles.
pub mod handle {
    /// The last photo taken.
    pub const PHOTO: u32 = 0xFFFF_C001;
    /// The current live-view frame.
    pub const LIVE_VIEW: u32 = 0xFFFF_C002;
}

/// Property codes seen in use.
pub mod prop {
    /// File format; values in [`super::file_format`]. Set with `SET_VALUE`.
    pub const FILE_FORMAT: u16 = 0x5004;
    /// Aperture as f-number × 100 (standard PTP code).
    pub const F_NUMBER: u16 = 0x5007;
    /// Shutter speed, see [`super::shutter_speed`]. Moved by steps with `CONTROL`.
    pub const SHUTTER_SPEED: u16 = 0xD20D;
    /// ISO; moved by steps with `CONTROL`.
    pub const ISO: u16 = 0xD21E;
    /// Shutter button, half press: 2 = down, 1 = up.
    pub const SHUTTER_HALF: u16 = 0xD2C1;
    /// Shutter button, full press: 2 = down, 1 = up.
    pub const SHUTTER_FULL: u16 = 0xD2C2;
    /// Shot state: 0 → 1 → `0x8001` → 0 on every shot; see [`super::photo_ready`].
    pub const SHOT_STATE: u16 = 0xD215;
}

/// Values of [`prop::FILE_FORMAT`] seen on the a6400.
pub mod file_format {
    pub const JPEG: u16 = 0x02;
    pub const RAW: u16 = 0x10;
}

/// Object formats of the photos read from [`handle::PHOTO`].
pub mod object_format {
    pub const JPEG: u16 = 0x3801;
    pub const RAW: u16 = 0xB101;
}

/// What a session sends, in order, before the body unlocks: operation and parameters.
pub const HANDSHAKE: [(u16, &[u32]); 5] = [
    (op::HANDSHAKE, &[1, 0, 0]),
    (op::HANDSHAKE, &[2, 0, 0]),
    (op::PROPERTY_CODES, &[0xC8]),
    (op::HANDSHAKE, &[3, 0, 0]),
    (op::PROPERTY_CODES, &[0xC8]),
];

/// One shot from the computer: `CONTROL` with each (property, value), in order, with
/// [`SHUTTER_STEP_GAP`] between steps.
pub const SHUTTER_PRESS: [(u16, i16); 4] = [
    (prop::SHUTTER_HALF, 2),
    (prop::SHUTTER_FULL, 2),
    (prop::SHUTTER_FULL, 1),
    (prop::SHUTTER_HALF, 1),
];

/// The wait between [`SHUTTER_PRESS`] steps. Sent back to back (about 40 ms in all) the
/// a6400 released before autofocus and did not shoot; 300 ms and 500 ms both shot.
pub const SHUTTER_STEP_GAP: std::time::Duration = std::time::Duration::from_millis(300);

/// Whether [`prop::SHOT_STATE`] says a new photo can be read from [`handle::PHOTO`]. Through
/// WPD this is how a shot is noticed: the `NEW_PHOTO` event did not arrive there.
pub fn photo_ready(shot_state: i128) -> bool {
    shot_state & 0x8000 != 0
}

/// The data phase of a `CONTROL` operation. ISO takes a 4-byte field with the value in its
/// low half; everything else seen takes 2 bytes.
pub fn control_data(code: u16, value: i16) -> Vec<u8> {
    let mut out = value.to_le_bytes().to_vec();
    if code == prop::ISO {
        out.extend_from_slice(&[0, 0]);
    }
    out
}

/// The data phase of a `SET_VALUE` operation for a 2-byte property such as the file format.
pub fn set_value_data(value: u16) -> [u8; 2] {
    value.to_le_bytes()
}

/// A shutter speed as (numerator, denominator) seconds: `0x0001_00FA` is 1/250 s.
pub fn shutter_speed(raw: u32) -> Option<(u16, u16)> {
    let numerator = u16::try_from(raw >> 16).ok()?;
    let denominator = u16::try_from(raw & 0xFFFF).ok()?;
    (numerator != 0 && denominator != 0).then_some((numerator, denominator))
}

/// The image inside a live-view data block: the block starts with the image's offset and
/// length (two little-endian `u32`), and the image ends the block.
pub fn live_view_image(data: &[u8]) -> Result<&[u8], PtpError> {
    let data = payload(data);
    let mut r = Reader::new(data);
    let offset = r.u32()? as usize;
    let len = r.u32()? as usize;
    let end = offset
        .checked_add(len)
        .filter(|&e| offset >= 8 && e <= data.len());
    let Some(end) = end else {
        return Err(PtpError::Truncated {
            at: offset,
            needed: len,
        });
    };
    Ok(&data[offset..end])
}

/// A property value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i128),
    Text(String),
}

impl Value {
    pub fn as_int(&self) -> Option<i128> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Text(_) => None,
        }
    }
}

/// Which values a property accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Form {
    None,
    Range { min: Value, max: Value, step: Value },
    List(Vec<Value>),
}

/// One entry of the [`op::ALL_SETTINGS`] block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub code: u16,
    /// PTP data type code (`0x0002` = u8, `0x0004` = u16, `0x0006` = u32, `0xFFFF` = string…).
    pub data_type: u16,
    /// 1 = set with `SET_VALUE`; 0 = read only, though some (ISO, shutter speed) still move
    /// by steps with `CONTROL`; `0x81` = a button for `CONTROL`.
    pub get_set: u8,
    /// A second byte Sony adds after `get_set` (0, 1 or 2); meaning to confirm.
    pub state: u8,
    pub default: Value,
    pub current: Value,
    pub form: Form,
}

/// The [`op::ALL_SETTINGS`] block: a count, four bytes we do not use, then one standard PTP
/// property description per entry with Sony's extra `state` byte after `get_set`.
pub struct Settings;

impl Settings {
    pub fn parse(bytes: &[u8]) -> Result<Vec<Property>, PtpError> {
        let mut r = Reader::new(payload(bytes));
        let count = r.u32()? as usize;
        if count > MAX_LIST {
            return Err(PtpError::ListTooLong(count));
        }
        r.u32()?;
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let code = r.u16()?;
            let data_type = r.u16()?;
            let get_set = r.u8()?;
            let state = r.u8()?;
            let default = value(&mut r, data_type)?;
            let current = value(&mut r, data_type)?;
            let form = match r.u8()? {
                0 => Form::None,
                1 => Form::Range {
                    min: value(&mut r, data_type)?,
                    max: value(&mut r, data_type)?,
                    step: value(&mut r, data_type)?,
                },
                2 => {
                    let n = usize::from(r.u16()?);
                    let mut list = Vec::with_capacity(n);
                    for _ in 0..n {
                        list.push(value(&mut r, data_type)?);
                    }
                    Form::List(list)
                }
                other => return Err(PtpError::UnknownForm(other)),
            };
            out.push(Property {
                code,
                data_type,
                get_set,
                state,
                default,
                current,
                form,
            });
        }
        Ok(out)
    }
}

fn value(r: &mut Reader<'_>, data_type: u16) -> Result<Value, PtpError> {
    Ok(Value::Int(match data_type {
        0x0001 => i128::from(r.u8()?.cast_signed()),
        0x0002 => i128::from(r.u8()?),
        0x0003 => i128::from(r.u16()?.cast_signed()),
        0x0004 => i128::from(r.u16()?),
        0x0005 => i128::from(r.u32()?.cast_signed()),
        0x0006 => i128::from(r.u32()?),
        0x0007 => i128::from(r.u64()?.cast_signed()),
        0x0008 => i128::from(r.u64()?),
        0xFFFF => return Ok(Value::Text(r.string()?)),
        other => return Err(PtpError::UnknownDataType(other)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-made settings block shaped like the a6400's (not copied from one): file format
    /// (u8 list), shutter speed (u32 range), exposure bias (i16 list) and a button.
    fn settings() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&4u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        // File format: u8, settable, default JPEG, current RAW, list of 3.
        b.extend_from_slice(&prop::FILE_FORMAT.to_le_bytes());
        b.extend_from_slice(&0x0002u16.to_le_bytes());
        b.extend_from_slice(&[1, 1, 0x02, 0x10, 2]);
        b.extend_from_slice(&3u16.to_le_bytes());
        b.extend_from_slice(&[0x02, 0x03, 0x10]);
        // Shutter speed: u32, read only, current 1/125, range.
        b.extend_from_slice(&prop::SHUTTER_SPEED.to_le_bytes());
        b.extend_from_slice(&0x0006u16.to_le_bytes());
        b.extend_from_slice(&[0, 1]);
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0x0001_007Du32.to_le_bytes());
        b.push(1);
        for v in [0u32, u32::MAX, 1] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        // Exposure bias: i16, list with a negative value.
        b.extend_from_slice(&0x5010u16.to_le_bytes());
        b.extend_from_slice(&0x0003u16.to_le_bytes());
        b.extend_from_slice(&[0, 2]);
        b.extend_from_slice(&0i16.to_le_bytes());
        b.extend_from_slice(&(-300i16).to_le_bytes());
        b.push(2);
        b.extend_from_slice(&2u16.to_le_bytes());
        b.extend_from_slice(&300i16.to_le_bytes());
        b.extend_from_slice(&(-300i16).to_le_bytes());
        // Half-press button: u16, 0x81, no form.
        b.extend_from_slice(&prop::SHUTTER_HALF.to_le_bytes());
        b.extend_from_slice(&0x0004u16.to_le_bytes());
        b.extend_from_slice(&[0x81, 1]);
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.push(0);
        b
    }

    #[test]
    fn settings_block_parses() -> Result<(), PtpError> {
        let p = Settings::parse(&settings())?;
        assert_eq!(p.len(), 4);
        assert_eq!(p[0].code, prop::FILE_FORMAT);
        assert_eq!(p[0].current, Value::Int(i128::from(file_format::RAW)));
        assert_eq!(
            p[0].form,
            Form::List(vec![Value::Int(2), Value::Int(3), Value::Int(0x10)])
        );
        let speed = p[1].current.as_int().and_then(|v| u32::try_from(v).ok());
        assert_eq!(speed.and_then(shutter_speed), Some((1, 125)));
        assert_eq!(p[2].current, Value::Int(-300));
        assert_eq!(p[3].get_set, 0x81);
        assert_eq!(p[3].form, Form::None);
        Ok(())
    }

    #[test]
    fn bad_settings_are_errors_not_panics() {
        let raw = settings();
        for cut in 0..raw.len() {
            assert!(Settings::parse(&raw[..cut]).is_err(), "cut at {cut}");
        }
        let mut huge = raw.clone();
        huge[0..4].copy_from_slice(&1_000_000_000u32.to_le_bytes());
        assert_eq!(
            Settings::parse(&huge),
            Err(PtpError::ListTooLong(1_000_000_000))
        );
        let mut bad_type = raw.clone();
        bad_type[10..12].copy_from_slice(&0x0042u16.to_le_bytes());
        assert_eq!(
            Settings::parse(&bad_type),
            Err(PtpError::UnknownDataType(0x42))
        );
        let mut bad_form = raw;
        bad_form[16] = 7;
        assert_eq!(Settings::parse(&bad_form), Err(PtpError::UnknownForm(7)));
    }

    #[test]
    fn live_view_image_is_bounded() -> Result<(), PtpError> {
        let mut block = 12u32.to_le_bytes().to_vec();
        block.extend_from_slice(&3u32.to_le_bytes());
        block.extend_from_slice(&[0xAA; 4]);
        block.extend_from_slice(&[0xFF, 0xD8, 0xD9]);
        assert_eq!(live_view_image(&block)?, &[0xFF, 0xD8, 0xD9]);
        block[4..8].copy_from_slice(&4u32.to_le_bytes());
        assert!(live_view_image(&block).is_err());
        block[0..4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(live_view_image(&block).is_err());
        block[0..4].copy_from_slice(&0u32.to_le_bytes());
        assert!(live_view_image(&block).is_err());
        assert!(live_view_image(&[1, 2, 3]).is_err());
        Ok(())
    }

    #[test]
    fn controls_and_values_are_encoded_as_observed() {
        assert_eq!(control_data(prop::SHUTTER_HALF, 2), vec![2, 0]);
        assert_eq!(control_data(prop::SHUTTER_SPEED, -1), vec![0xFF, 0xFF]);
        assert_eq!(control_data(prop::ISO, -1), vec![0xFF, 0xFF, 0, 0]);
        assert_eq!(set_value_data(file_format::JPEG), [0x02, 0x00]);
        assert_eq!(shutter_speed(0x0001_00FA), Some((1, 250)));
        assert_eq!(shutter_speed(0x001E_000A), Some((30, 10)));
        assert_eq!(shutter_speed(0), None);
        assert_eq!(HANDSHAKE.len(), 5);
        assert_eq!(SHUTTER_PRESS.first(), Some(&(prop::SHUTTER_HALF, 2)));
        assert_eq!(SHUTTER_PRESS.last(), Some(&(prop::SHUTTER_HALF, 1)));
        assert!(photo_ready(0x8001));
        assert!(!photo_ready(0));
        assert!(!photo_ready(1));
    }
}
