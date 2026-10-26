// SPDX-License-Identifier: GPL-3.0-only
//! PTP (ISO 15740, "Picture Transfer Protocol"), the language cameras speak over USB: packing
//! an operation request and reading what the camera sends back. Pure Rust, no I/O, so it is
//! tested on recorded responses without a camera (ADR-0026).
//!
//! Everything a camera sends is untrusted input (SECURITY.md §3): every length and count is
//! checked against the bytes actually there, and lists are capped, so a bad answer is an
//! error, never a panic or a huge allocation.

use thiserror::Error;

/// Standard operation codes used by Fotollection (ISO 15740 §10.4).
pub mod op {
    pub const GET_DEVICE_INFO: u16 = 0x1001;
    pub const OPEN_SESSION: u16 = 0x1002;
    pub const CLOSE_SESSION: u16 = 0x1003;
    pub const GET_STORAGE_IDS: u16 = 0x1004;
    pub const GET_OBJECT_INFO: u16 = 0x1008;
    pub const GET_OBJECT: u16 = 0x1009;
    pub const INITIATE_CAPTURE: u16 = 0x100E;
    pub const GET_DEVICE_PROP_DESC: u16 = 0x1014;
    pub const GET_DEVICE_PROP_VALUE: u16 = 0x1015;
}

/// Response codes (ISO 15740 §11).
pub mod rc {
    pub const OK: u16 = 0x2001;
    pub const GENERAL_ERROR: u16 = 0x2002;
    pub const OPERATION_NOT_SUPPORTED: u16 = 0x2005;
    pub const DEVICE_BUSY: u16 = 0x2019;
}

/// Event codes (ISO 15740 §12).
pub mod ev {
    pub const OBJECT_ADDED: u16 = 0x4002;
    pub const CAPTURE_COMPLETE: u16 = 0x400D;
}

/// Container types in the 12-byte header.
const COMMAND: u16 = 1;
const DATA: u16 = 2;
const RESPONSE: u16 = 3;
const EVENT: u16 = 4;
const HEADER: usize = 12;

/// No list in a dataset is longer than this; a real camera declares a few hundred codes.
pub(crate) const MAX_LIST: usize = 4096;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PtpError {
    #[error("the camera's answer is cut short (needed {needed} bytes at offset {at})")]
    Truncated { at: usize, needed: usize },
    #[error("the camera's answer is not a PTP {expected} container")]
    NotContainer { expected: &'static str },
    #[error("a list in the camera's answer is too long ({0} entries)")]
    ListTooLong(usize),
    #[error("the camera's answer uses an unknown data type {0:#06x}")]
    UnknownDataType(u16),
    #[error("the camera's answer uses an unknown form {0}")]
    UnknownForm(u8),
}

/// The 12-byte header plus up to five parameters of a command request.
pub fn command(code: u16, transaction: u32, params: &[u32]) -> Vec<u8> {
    let params = &params[..params.len().min(5)];
    let len = HEADER + 4 * params.len();
    let mut out = Vec::with_capacity(len);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(u32::MAX).to_le_bytes());
    out.extend_from_slice(&COMMAND.to_le_bytes());
    out.extend_from_slice(&code.to_le_bytes());
    out.extend_from_slice(&transaction.to_le_bytes());
    for p in params {
        out.extend_from_slice(&p.to_le_bytes());
    }
    out
}

/// An event container rebuilt from its code and parameters, for transports that hand events
/// over already taken apart (WPD). Transaction `0xFFFFFFFF`, as the a6400 sends its own.
pub fn event_container(code: u16, params: &[u32]) -> Vec<u8> {
    let params = &params[..params.len().min(5)];
    let len = HEADER + 4 * params.len();
    let mut out = Vec::with_capacity(len);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(u32::MAX).to_le_bytes());
    out.extend_from_slice(&EVENT.to_le_bytes());
    out.extend_from_slice(&code.to_le_bytes());
    out.extend_from_slice(&u32::MAX.to_le_bytes());
    for p in params {
        out.extend_from_slice(&p.to_le_bytes());
    }
    out
}

/// A response or event container: its code and parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub code: u16,
    pub transaction: u32,
    pub params: Vec<u32>,
}

impl Container {
    pub fn is_ok(&self) -> bool {
        self.code == rc::OK
    }
}

fn container(bytes: &[u8], kind: u16, expected: &'static str) -> Result<Container, PtpError> {
    let mut r = Reader::new(bytes);
    let len = r.u32()? as usize;
    let ty = r.u16()?;
    if ty != kind || len < HEADER || len > bytes.len() {
        return Err(PtpError::NotContainer { expected });
    }
    let code = r.u16()?;
    let transaction = r.u32()?;
    let mut params = Vec::new();
    while r.at + 4 <= len && params.len() < 5 {
        params.push(r.u32()?);
    }
    Ok(Container {
        code,
        transaction,
        params,
    })
}

/// Reads a response container.
pub fn response(bytes: &[u8]) -> Result<Container, PtpError> {
    container(bytes, RESPONSE, "response")
}

/// Reads an event container.
pub fn event(bytes: &[u8]) -> Result<Container, PtpError> {
    container(bytes, EVENT, "event")
}

/// The data phase without its container header, when the transport left the header on.
pub fn payload(bytes: &[u8]) -> &[u8] {
    let mut r = Reader::new(bytes);
    match (r.u32(), r.u16()) {
        (Ok(len), Ok(DATA)) if len as usize == bytes.len() && bytes.len() >= HEADER => {
            &bytes[HEADER..]
        }
        _ => bytes,
    }
}

/// What a camera says about itself (`GetDeviceInfo`, ISO 15740 §5.5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub standard_version: u16,
    pub vendor_extension_id: u32,
    pub vendor_extension_version: u16,
    pub vendor_extension_desc: String,
    pub functional_mode: u16,
    pub operations: Vec<u16>,
    pub events: Vec<u16>,
    pub properties: Vec<u16>,
    pub capture_formats: Vec<u16>,
    pub image_formats: Vec<u16>,
    pub manufacturer: String,
    pub model: String,
    pub device_version: String,
    pub serial_number: String,
}

impl DeviceInfo {
    pub fn parse(bytes: &[u8]) -> Result<Self, PtpError> {
        let mut r = Reader::new(payload(bytes));
        Ok(Self {
            standard_version: r.u16()?,
            vendor_extension_id: r.u32()?,
            vendor_extension_version: r.u16()?,
            vendor_extension_desc: r.string()?,
            functional_mode: r.u16()?,
            operations: r.u16_list()?,
            events: r.u16_list()?,
            properties: r.u16_list()?,
            capture_formats: r.u16_list()?,
            image_formats: r.u16_list()?,
            manufacturer: r.string()?,
            model: r.string()?,
            device_version: r.string()?,
            serial_number: r.string()?,
        })
    }

    /// Whether the camera declares an operation.
    pub fn supports(&self, code: u16) -> bool {
        self.operations.contains(&code)
    }
}

/// What a camera says about one file (`GetObjectInfo`, ISO 15740 §5.5.2); only the fields
/// Fotollection uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfo {
    pub storage_id: u32,
    pub format: u16,
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub filename: String,
}

impl ObjectInfo {
    pub fn parse(bytes: &[u8]) -> Result<Self, PtpError> {
        let mut r = Reader::new(payload(bytes));
        let storage_id = r.u32()?;
        let format = r.u16()?;
        r.u16()?; // protection
        let size = r.u32()?;
        r.take(2 + 4 + 4 + 4)?; // thumbnail format, size, width, height
        let width = r.u32()?;
        let height = r.u32()?;
        r.take(4 + 4 + 2 + 4 + 4)?; // bit depth, parent, association type and desc, sequence
        let filename = r.string()?;
        Ok(Self {
            storage_id,
            format,
            size,
            width,
            height,
            filename,
        })
    }
}

/// The vendor extension IDs PTP assigns (ISO 15740 and PIMA's registry), for printing.
pub fn vendor_name(id: u32) -> &'static str {
    match id {
        0x0000_0006 => "Microsoft (MTP)",
        0x0000_000B => "Canon",
        0x0000_000A => "Nikon",
        0x0000_0011 => "Sony",
        0x0000_001C => "Panasonic",
        0x0000_000E => "Fujifilm",
        0x0000_0001 => "Kodak",
        _ => "unknown",
    }
}

/// A readable name for a standard operation code, for printing.
pub fn operation_name(code: u16) -> Option<&'static str> {
    Some(match code {
        0x1001 => "GetDeviceInfo",
        0x1002 => "OpenSession",
        0x1003 => "CloseSession",
        0x1004 => "GetStorageIDs",
        0x1005 => "GetStorageInfo",
        0x1006 => "GetNumObjects",
        0x1007 => "GetObjectHandles",
        0x1008 => "GetObjectInfo",
        0x1009 => "GetObject",
        0x100A => "GetThumb",
        0x100B => "DeleteObject",
        0x100C => "SendObjectInfo",
        0x100D => "SendObject",
        0x100E => "InitiateCapture",
        0x100F => "FormatStore",
        0x1010 => "ResetDevice",
        0x1011 => "SelfTest",
        0x1012 => "SetObjectProtection",
        0x1013 => "PowerDown",
        0x1014 => "GetDevicePropDesc",
        0x1015 => "GetDevicePropValue",
        0x1016 => "SetDevicePropValue",
        0x1017 => "ResetDevicePropValue",
        0x1018 => "TerminateOpenCapture",
        0x1019 => "MoveObject",
        0x101A => "CopyObject",
        0x101B => "GetPartialObject",
        0x101C => "InitiateOpenCapture",
        _ => return None,
    })
}

/// A readable name for a standard event code, for printing.
pub fn event_name(code: u16) -> Option<&'static str> {
    Some(match code {
        0x4001 => "CancelTransaction",
        0x4002 => "ObjectAdded",
        0x4003 => "ObjectRemoved",
        0x4004 => "StoreAdded",
        0x4005 => "StoreRemoved",
        0x4006 => "DevicePropChanged",
        0x4007 => "ObjectInfoChanged",
        0x4008 => "DeviceInfoChanged",
        0x4009 => "RequestObjectTransfer",
        0x400A => "StoreFull",
        0x400B => "DeviceReset",
        0x400C => "StorageInfoChanged",
        0x400D => "CaptureComplete",
        0x400E => "UnreportedStatus",
        _ => return None,
    })
}

/// Little-endian reader over untrusted bytes, shared with the maker drivers.
pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    pub(crate) at: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], PtpError> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.bytes.len());
        let Some(end) = end else {
            return Err(PtpError::Truncated {
                at: self.at,
                needed: n,
            });
        };
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, PtpError> {
        Ok(self.take(1)?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16, PtpError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, PtpError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, PtpError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// A PTP string: a count of UTF-16 code units including the final NUL, then the units.
    pub(crate) fn string(&mut self) -> Result<String, PtpError> {
        let n = usize::from(self.u8()?);
        let raw = self.take(2 * n)?;
        let units: Vec<u16> = raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .take_while(|&u| u != 0)
            .collect();
        Ok(String::from_utf16_lossy(&units))
    }

    pub(crate) fn u16_list(&mut self) -> Result<Vec<u16>, PtpError> {
        let n = self.u32()? as usize;
        if n > MAX_LIST {
            return Err(PtpError::ListTooLong(n));
        }
        let raw = self.take(2 * n)?;
        Ok(raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(s: &str) -> Vec<u8> {
        if s.is_empty() {
            return vec![0];
        }
        let units: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
        let mut out = vec![u8::try_from(units.len()).unwrap_or(u8::MAX)];
        for u in units {
            out.extend_from_slice(&u.to_le_bytes());
        }
        out
    }

    fn list(codes: &[u16]) -> Vec<u8> {
        let mut out = u32::try_from(codes.len())
            .unwrap_or(0)
            .to_le_bytes()
            .to_vec();
        for c in codes {
            out.extend_from_slice(&c.to_le_bytes());
        }
        out
    }

    /// A hand-made `DeviceInfo`, shaped like a camera's (not copied from one).
    fn device_info() -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(&100u16.to_le_bytes());
        d.extend_from_slice(&0x11u32.to_le_bytes());
        d.extend_from_slice(&100u16.to_le_bytes());
        d.extend(string("Test extension"));
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend(list(&[0x1001, 0x1002, 0x100E, 0x9201]));
        d.extend(list(&[0x4002, 0xC201]));
        d.extend(list(&[0x5001]));
        d.extend(list(&[]));
        d.extend(list(&[0x3801, 0xB101]));
        d.extend(string("Maker"));
        d.extend(string("Model 1"));
        d.extend(string("1.00"));
        d.extend(string(""));
        d
    }

    #[test]
    fn command_has_header_and_params() {
        let c = command(op::OPEN_SESSION, 7, &[1]);
        assert_eq!(c.len(), 16);
        assert_eq!(&c[0..4], &16u32.to_le_bytes());
        assert_eq!(&c[4..6], &COMMAND.to_le_bytes());
        assert_eq!(&c[6..8], &op::OPEN_SESSION.to_le_bytes());
        assert_eq!(&c[8..12], &7u32.to_le_bytes());
        assert_eq!(&c[12..16], &1u32.to_le_bytes());
        // At most five parameters.
        assert_eq!(command(op::GET_OBJECT, 1, &[0; 9]).len(), 32);
    }

    #[test]
    fn device_info_parses_with_or_without_a_data_header() -> Result<(), PtpError> {
        let raw = device_info();
        let info = DeviceInfo::parse(&raw)?;
        assert_eq!(info.vendor_extension_id, 0x11);
        assert_eq!(info.vendor_extension_desc, "Test extension");
        assert_eq!(info.operations, vec![0x1001, 0x1002, 0x100E, 0x9201]);
        assert!(info.supports(op::INITIATE_CAPTURE));
        assert_eq!(info.model, "Model 1");
        assert_eq!(info.serial_number, "");

        let mut wrapped = u32::try_from(raw.len() + 12)
            .unwrap_or(0)
            .to_le_bytes()
            .to_vec();
        wrapped.extend_from_slice(&DATA.to_le_bytes());
        wrapped.extend_from_slice(&op::GET_DEVICE_INFO.to_le_bytes());
        wrapped.extend_from_slice(&1u32.to_le_bytes());
        wrapped.extend(raw);
        assert_eq!(DeviceInfo::parse(&wrapped)?, info);
        Ok(())
    }

    #[test]
    fn bad_answers_are_errors_not_panics() {
        let raw = device_info();
        for cut in 0..raw.len() {
            assert!(DeviceInfo::parse(&raw[..cut]).is_err(), "cut at {cut}");
        }
        let mut huge = raw[..raw.len() - 1].to_vec();
        // A list that claims a billion entries.
        let at = 2 + 4 + 2 + 31 + 2;
        huge.splice(at..at + 4, 1_000_000_000u32.to_le_bytes());
        assert!(DeviceInfo::parse(&huge).is_err());
        assert!(response(&[1, 2, 3]).is_err());
        assert!(response(&command(op::OPEN_SESSION, 1, &[1])).is_err());
    }

    /// A hand-made `ObjectInfo` for a 6000 × 4000 RAW of 24 MB.
    fn object_info() -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0001_0000u32.to_le_bytes());
        d.extend_from_slice(&0xB101u16.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&24_000_000u32.to_le_bytes());
        d.extend_from_slice(&[0; 14]);
        d.extend_from_slice(&6000u32.to_le_bytes());
        d.extend_from_slice(&4000u32.to_le_bytes());
        d.extend_from_slice(&[0; 18]);
        d.extend(string("DSC00001.ARW"));
        d.extend(string(""));
        d
    }

    #[test]
    fn object_info_parses_and_bad_answers_are_errors() -> Result<(), PtpError> {
        let raw = object_info();
        let info = ObjectInfo::parse(&raw)?;
        assert_eq!(info.format, 0xB101);
        assert_eq!(info.size, 24_000_000);
        assert_eq!((info.width, info.height), (6000, 4000));
        assert_eq!(info.filename, "DSC00001.ARW");
        let name_end = raw.len() - 1;
        for cut in 0..name_end {
            assert!(ObjectInfo::parse(&raw[..cut]).is_err(), "cut at {cut}");
        }
        Ok(())
    }

    #[test]
    fn response_and_event_containers() -> Result<(), PtpError> {
        let mut r = 16u32.to_le_bytes().to_vec();
        r.extend_from_slice(&RESPONSE.to_le_bytes());
        r.extend_from_slice(&rc::OK.to_le_bytes());
        r.extend_from_slice(&3u32.to_le_bytes());
        r.extend_from_slice(&0xFFFF_C001u32.to_le_bytes());
        let c = response(&r)?;
        assert!(c.is_ok());
        assert_eq!(c.params, vec![0xFFFF_C001]);

        let mut e = 16u32.to_le_bytes().to_vec();
        e.extend_from_slice(&EVENT.to_le_bytes());
        e.extend_from_slice(&ev::OBJECT_ADDED.to_le_bytes());
        e.extend_from_slice(&0u32.to_le_bytes());
        e.extend_from_slice(&42u32.to_le_bytes());
        assert_eq!(event(&e)?.params, vec![42]);

        let rebuilt = event(&event_container(0xC201, &[0xFFFF_C001]))?;
        assert_eq!(rebuilt.code, 0xC201);
        assert_eq!(rebuilt.params, vec![0xFFFF_C001]);
        assert_eq!(event(&event_container(0xC203, &[0; 9]))?.params.len(), 5);
        Ok(())
    }
}
