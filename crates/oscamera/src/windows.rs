// SPDX-License-Identifier: GPL-3.0-only
//! The Windows transport: Windows Portable Devices (WPD) finds cameras and carries any PTP
//! operation, standard or a maker's, through its MTP extension commands (ADR-0026). The camera
//! keeps Windows's own driver (`wpdmtp.inf`): nothing is installed, and the session is the one
//! Windows opened when the camera was plugged in.
//!
//! WPD calls block. Each open camera gets its own thread, which owns every COM object of that
//! camera; a call waits for that thread's answer with a timeout, so a camera that stops
//! answering never hangs the caller (SECURITY.md). No COM object crosses threads.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use thiserror::Error;
use windows::Win32::Devices::PortableDevices::{
    IPortableDevice, IPortableDeviceEventCallback, IPortableDeviceEventCallback_Impl,
    IPortableDeviceManager, IPortableDevicePropVariantCollection, IPortableDeviceValues,
    PortableDeviceFTM, PortableDeviceManager, PortableDevicePropVariantCollection,
    PortableDeviceValues, WPD_CLIENT_DESIRED_ACCESS, WPD_CLIENT_MAJOR_VERSION,
    WPD_CLIENT_MINOR_VERSION, WPD_CLIENT_NAME, WPD_CLIENT_REVISION,
    WPD_CLIENT_SECURITY_QUALITY_OF_SERVICE, WPD_COMMAND_MTP_EXT_END_DATA_TRANSFER,
    WPD_COMMAND_MTP_EXT_EXECUTE_COMMAND_WITH_DATA_TO_READ,
    WPD_COMMAND_MTP_EXT_EXECUTE_COMMAND_WITH_DATA_TO_WRITE, WPD_COMMAND_MTP_EXT_READ_DATA,
    WPD_COMMAND_MTP_EXT_WRITE_DATA, WPD_DEVICE_OBJECT_ID, WPD_DEVICE_PROTOCOL,
    WPD_EVENT_DEVICE_REMOVED, WPD_EVENT_MTP_VENDOR_EXTENDED_EVENTS, WPD_EVENT_PARAMETER_EVENT_ID,
    WPD_PROPERTY_COMMON_COMMAND_CATEGORY, WPD_PROPERTY_COMMON_COMMAND_ID,
    WPD_PROPERTY_COMMON_HRESULT, WPD_PROPERTY_MTP_EXT_EVENT_PARAMS,
    WPD_PROPERTY_MTP_EXT_OPERATION_CODE, WPD_PROPERTY_MTP_EXT_OPERATION_PARAMS,
    WPD_PROPERTY_MTP_EXT_OPTIMAL_TRANSFER_BUFFER_SIZE, WPD_PROPERTY_MTP_EXT_RESPONSE_CODE,
    WPD_PROPERTY_MTP_EXT_RESPONSE_PARAMS, WPD_PROPERTY_MTP_EXT_TRANSFER_CONTEXT,
    WPD_PROPERTY_MTP_EXT_TRANSFER_DATA, WPD_PROPERTY_MTP_EXT_TRANSFER_NUM_BYTES_READ,
    WPD_PROPERTY_MTP_EXT_TRANSFER_NUM_BYTES_TO_READ,
    WPD_PROPERTY_MTP_EXT_TRANSFER_NUM_BYTES_TO_WRITE,
    WPD_PROPERTY_MTP_EXT_TRANSFER_TOTAL_DATA_SIZE,
};
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, PROPERTYKEY};
use windows::Win32::System::Com::StructuredStorage::{
    PROPVARIANT, PropVariantClear, PropVariantToUInt32,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
    CoUninitialize,
};
use windows::Win32::System::Variant::VT_UI4;
use windows::core::{GUID, HSTRING, PCWSTR, PWSTR, implement};

use crate::ptp::{Container, event_container};

/// No object read from a camera is larger than this; a RAW file from a 61 MP body is ~120 MB
/// uncompressed, the a6400's is 25 MB.
const MAX_OBJECT: u64 = 256 * 1024 * 1024;

/// `SECURITY_IMPERSONATION`, the quality of service WPD clients are documented to ask for.
const SECURITY_IMPERSONATION: u32 = 0x0002_0000;

#[derive(Debug, Error)]
pub enum CameraError {
    #[error("the camera did not answer within {0:?}")]
    Timeout(Duration),
    #[error("the camera refused: {0}")]
    Refused(String),
    #[error("the camera is not open, or stopped answering earlier")]
    Closed,
    #[error("the camera's answer is not PTP: {0}")]
    Ptp(#[from] crate::ptp::PtpError),
}

impl From<windows::core::Error> for CameraError {
    fn from(e: windows::core::Error) -> Self {
        Self::Refused(format!("{} ({})", e.message(), e.code()))
    }
}

/// Something the camera reported while a session is open.
#[derive(Debug, Clone)]
pub enum Event {
    /// A PTP event container, rebuilt from what WPD hands over.
    Ptp(Vec<u8>),
    /// The camera was unplugged or turned off.
    Removed,
}

/// An event with the moment it arrived.
#[derive(Debug, Clone)]
pub struct Timed {
    pub at: Instant,
    pub event: Event,
}

/// What a PTP operation brought back.
#[derive(Debug, Clone)]
pub struct Reply {
    /// The data phase, without a container header.
    pub data: Vec<u8>,
    pub response: Container,
}

/// Waits; WPD delivers events on its own threads, so there is no run loop to spin. Kept so
/// callers read the same on macOS and Windows.
pub fn pump(duration: Duration) {
    thread::sleep(duration);
}

/// Initialises COM (multithreaded) for the calling thread.
fn com_init() -> bool {
    // SAFETY: initialises COM for this thread; repeated calls are allowed, and a thread
    // already in another apartment returns an error that is reported, not ignored.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok()
}

fn create<T: windows::core::Interface>(class: &GUID) -> windows::core::Result<T> {
    // SAFETY: standard in-process creation of a WPD class.
    unsafe { CoCreateInstance(class, None, CLSCTX_INPROC_SERVER) }
}

/// Takes a string WPD allocated, and frees it.
fn take_string(p: PWSTR) -> String {
    if p.is_null() {
        return String::new();
    }
    // SAFETY: WPD returns a NUL-terminated string from `CoTaskMemAlloc`; it is read once,
    // then freed, and not used again.
    unsafe {
        let s = p.to_string().unwrap_or_default();
        CoTaskMemFree(Some(p.0 as *const _));
        s
    }
}

/// Looks for cameras and other WPD devices connected to this PC.
pub struct Browser {
    manager: IPortableDeviceManager,
}

impl Browser {
    pub fn start() -> Result<Self, CameraError> {
        com_init();
        Ok(Self {
            manager: create(&PortableDeviceManager)?,
        })
    }

    /// The devices that speak PTP or MTP, waiting up to `timeout` for the first one.
    pub fn cameras(&self, timeout: Duration) -> Vec<Camera> {
        let start = Instant::now();
        loop {
            let found: Vec<Camera> = self
                .device_ids()
                .into_iter()
                .map(|id| self.describe(id))
                .filter(|c| c.protocol.contains("PTP") || c.protocol.contains("MTP"))
                .collect();
            if !found.is_empty() || start.elapsed() >= timeout {
                return found;
            }
            thread::sleep(Duration::from_millis(250));
        }
    }

    fn device_ids(&self) -> Vec<String> {
        // SAFETY: asks WPD to look again; no arguments.
        let _ = unsafe { self.manager.RefreshDeviceList() };
        let mut count = 0u32;
        // SAFETY: a null array asks only for the count.
        if unsafe {
            self.manager
                .GetDevices(std::ptr::null_mut(), &raw mut count)
        }
        .is_err()
            || count == 0
        {
            return Vec::new();
        }
        let mut ids = vec![PWSTR::null(); count as usize];
        // SAFETY: `ids` holds exactly `count` slots; WPD fills at most `count` and updates it.
        if unsafe { self.manager.GetDevices(ids.as_mut_ptr(), &raw mut count) }.is_err() {
            return Vec::new();
        }
        ids.into_iter()
            .take(count as usize)
            .map(take_string)
            .collect()
    }

    fn describe(&self, id: String) -> Camera {
        let wide = HSTRING::from(id.as_str());
        let name = self.manager_string(&wide, true);
        let manufacturer = self.manager_string(&wide, false);
        let protocol = device_protocol(&wide).unwrap_or_default();
        Camera {
            id,
            name,
            manufacturer,
            protocol,
            worker: None,
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn manager_string(&self, id: &HSTRING, name: bool) -> String {
        let mut len = 0u32;
        let get = |buf: PWSTR, len: *mut u32| {
            // SAFETY: `buf` is null (asking for the length) or holds `*len` UTF-16 units.
            unsafe {
                if name {
                    self.manager
                        .GetDeviceFriendlyName(PCWSTR(id.as_ptr()), buf, len)
                } else {
                    self.manager
                        .GetDeviceManufacturer(PCWSTR(id.as_ptr()), buf, len)
                }
            }
        };
        if get(PWSTR::null(), &raw mut len).is_err() || len == 0 || len > 1024 {
            return String::new();
        }
        let mut buf = vec![0u16; len as usize];
        if get(PWSTR(buf.as_mut_ptr()), &raw mut len).is_err() {
            return String::new();
        }
        let end = buf.iter().position(|&u| u == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..end])
    }
}

/// Opens a device briefly to read its protocol ("PTP: 1.00", "MTP: 1.00").
fn device_protocol(id: &HSTRING) -> windows::core::Result<String> {
    let device = open_device(id)?;
    // SAFETY: reads one property of the device object; the device is open.
    let protocol = unsafe {
        let props = device.Content()?.Properties()?;
        let values = props.GetValues(WPD_DEVICE_OBJECT_ID, None)?;
        take_string(values.GetStringValue(&WPD_DEVICE_PROTOCOL)?)
    };
    // SAFETY: closes the handle opened above.
    let _ = unsafe { device.Close() };
    Ok(protocol)
}

fn open_device(id: &HSTRING) -> windows::core::Result<IPortableDevice> {
    let device: IPortableDevice = create(&PortableDeviceFTM)?;
    let client: IPortableDeviceValues = create(&PortableDeviceValues)?;
    // SAFETY: fills the client description WPD asks for, then opens the device by its ID.
    unsafe {
        client.SetStringValue(&WPD_CLIENT_NAME, &HSTRING::from("Fotollection"))?;
        client.SetUnsignedIntegerValue(&WPD_CLIENT_MAJOR_VERSION, 0)?;
        client.SetUnsignedIntegerValue(&WPD_CLIENT_MINOR_VERSION, 7)?;
        client.SetUnsignedIntegerValue(&WPD_CLIENT_REVISION, 0)?;
        client.SetUnsignedIntegerValue(
            &WPD_CLIENT_SECURITY_QUALITY_OF_SERVICE,
            SECURITY_IMPERSONATION,
        )?;
        client.SetUnsignedIntegerValue(
            &WPD_CLIENT_DESIRED_ACCESS,
            GENERIC_READ.0 | GENERIC_WRITE.0,
        )?;
        device.Open(PCWSTR(id.as_ptr()), &client)?;
    }
    Ok(device)
}

/// A WPD device found by the browser.
pub struct Camera {
    id: String,
    name: String,
    manufacturer: String,
    protocol: String,
    worker: Option<Worker>,
    events: Arc<Mutex<Vec<Timed>>>,
}

type Answer = Result<Reply, CameraError>;

enum Job {
    Send {
        code: u16,
        params: Vec<u32>,
        out: Option<Vec<u8>>,
        reply: Sender<Answer>,
    },
}

struct Worker {
    jobs: Option<Sender<Job>>,
    thread: Option<JoinHandle<()>>,
    /// Set when a call timed out: the thread may still be stuck inside WPD.
    stuck: bool,
}

impl Camera {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn product_kind(&self) -> String {
        self.manufacturer.clone()
    }

    /// The protocol WPD reports, for example "PTP: 1.00".
    pub fn capabilities(&self) -> Vec<String> {
        vec![self.protocol.clone()]
    }

    /// Opens the device on its own thread and starts collecting events.
    pub fn open(&mut self, timeout: Duration) -> Result<(), CameraError> {
        let (jobs, inbox) = mpsc::channel::<Job>();
        let (ready, started) = mpsc::channel::<Result<(), CameraError>>();
        let id = self.id.clone();
        let events = self.events.clone();
        let thread = thread::Builder::new()
            .name("oscamera-wpd".into())
            .spawn(move || serve(&id, &events, &inbox, &ready))
            .map_err(|e| CameraError::Refused(e.to_string()))?;
        let mut worker = Worker {
            jobs: Some(jobs),
            thread: Some(thread),
            stuck: false,
        };
        match started.recv_timeout(timeout) {
            Ok(Ok(())) => {
                self.worker = Some(worker);
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                worker.stuck = true;
                drop(worker);
                Err(CameraError::Timeout(timeout))
            }
        }
    }

    /// Sends one PTP operation and waits for its answer. Without `out`, the camera may send a
    /// data phase back; with `out`, those bytes are the data phase sent to the camera.
    pub fn send(
        &mut self,
        code: u16,
        params: &[u32],
        out: Option<&[u8]>,
        timeout: Duration,
    ) -> Result<Reply, CameraError> {
        let worker = self.worker.as_mut().ok_or(CameraError::Closed)?;
        if worker.stuck {
            return Err(CameraError::Closed);
        }
        let (reply, answer) = mpsc::channel();
        let job = Job::Send {
            code,
            params: params.iter().copied().take(5).collect(),
            out: out.map(<[u8]>::to_vec),
            reply,
        };
        worker
            .jobs
            .as_ref()
            .ok_or(CameraError::Closed)?
            .send(job)
            .map_err(|_| CameraError::Closed)?;
        match answer.recv_timeout(timeout) {
            Ok(a) => a,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                worker.stuck = true;
                Err(CameraError::Timeout(timeout))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(CameraError::Closed),
        }
    }

    /// The events received since the last call.
    pub fn take_events(&self) -> Vec<Timed> {
        self.events
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Closing the channel ends the thread's loop; it then unregisters and closes.
        self.jobs = None;
        if let Some(t) = self.thread.take()
            && !self.stuck
        {
            let _ = t.join();
        }
    }
}

/// The camera's thread: opens the device, registers for events, then runs jobs until the
/// channel closes.
fn serve(
    id: &str,
    events: &Arc<Mutex<Vec<Timed>>>,
    inbox: &Receiver<Job>,
    ready: &Sender<Result<(), CameraError>>,
) {
    let initialised = com_init();
    let opened = (|| -> windows::core::Result<(IPortableDevice, PWSTR)> {
        let device = open_device(&HSTRING::from(id))?;
        let sink: IPortableDeviceEventCallback = EventSink {
            events: events.clone(),
        }
        .into();
        // SAFETY: registers `sink`, which COM keeps alive until `Unadvise`.
        let cookie = unsafe { device.Advise(0, &sink, None)? };
        Ok((device, cookie))
    })();
    let (device, cookie) = match opened {
        Ok(v) => v,
        Err(e) => {
            let _ = ready.send(Err(e.into()));
            return;
        }
    };
    let _ = ready.send(Ok(()));
    while let Ok(job) = inbox.recv() {
        match job {
            Job::Send {
                code,
                params,
                out,
                reply,
            } => {
                let _ = reply.send(execute(&device, code, &params, out.as_deref()));
            }
        }
    }
    // SAFETY: unregisters the sink with the cookie Advise returned, closes the device, then
    // releases the cookie string WPD allocated.
    unsafe {
        let _ = device.Unadvise(PCWSTR(cookie.0));
        let _ = device.Close();
        CoTaskMemFree(Some(cookie.0 as *const _));
    }
    drop(device);
    if initialised {
        // SAFETY: balances this thread's successful `CoInitializeEx`, after its last COM use.
        unsafe { CoUninitialize() };
    }
}

/// A command for WPD: its category and ID.
fn command(key: &PROPERTYKEY) -> windows::core::Result<IPortableDeviceValues> {
    let v: IPortableDeviceValues = create(&PortableDeviceValues)?;
    // SAFETY: sets two values on a fresh collection.
    unsafe {
        v.SetGuidValue(&WPD_PROPERTY_COMMON_COMMAND_CATEGORY, &raw const key.fmtid)?;
        v.SetUnsignedIntegerValue(&WPD_PROPERTY_COMMON_COMMAND_ID, key.pid)?;
    }
    Ok(v)
}

/// Sends a command and turns a failure WPD reports inside the results into an error.
fn run(
    device: &IPortableDevice,
    values: &IPortableDeviceValues,
) -> windows::core::Result<IPortableDeviceValues> {
    // SAFETY: the device is open; `values` is a complete command.
    unsafe {
        let results = device.SendCommand(0, values)?;
        results.GetErrorValue(&WPD_PROPERTY_COMMON_HRESULT)?.ok()?;
        Ok(results)
    }
}

fn propvariant_u32(value: u32) -> PROPVARIANT {
    let mut pv = PROPVARIANT::default();
    // SAFETY: a zeroed PROPVARIANT; sets the type tag and the member it selects.
    unsafe {
        let inner = &mut *pv.Anonymous.Anonymous;
        inner.vt = VT_UI4;
        inner.Anonymous.ulVal = value;
    }
    pv
}

fn params_collection(
    params: &[u32],
) -> windows::core::Result<IPortableDevicePropVariantCollection> {
    let c: IPortableDevicePropVariantCollection = create(&PortableDevicePropVariantCollection)?;
    for &p in params {
        // SAFETY: the collection copies the value; a VT_UI4 owns nothing to free.
        unsafe { c.Add(&propvariant_u32(p))? };
    }
    Ok(c)
}

fn collection_u32(c: &IPortableDevicePropVariantCollection) -> Vec<u32> {
    let mut n = 0u32;
    // SAFETY: WPD writes the count into `n`.
    if unsafe { c.GetCount(std::ptr::from_mut(&mut n).cast_const()) }.is_err() {
        return Vec::new();
    }
    (0..n.min(16))
        .filter_map(|i| {
            let mut pv = PROPVARIANT::default();
            // SAFETY: `i` is below the count; the value is converted, then cleared once.
            unsafe {
                c.GetAt(i, &raw const pv).ok()?;
                let v = PropVariantToUInt32(&raw const pv).ok();
                let _ = PropVariantClear(&raw mut pv);
                v
            }
        })
        .collect()
}

fn response(results: &IPortableDeviceValues) -> windows::core::Result<Container> {
    // SAFETY: reads values from a results collection WPD returned.
    unsafe {
        let code = results.GetUnsignedIntegerValue(&WPD_PROPERTY_MTP_EXT_RESPONSE_CODE)?;
        let params = results
            .GetIPortableDevicePropVariantCollectionValue(&WPD_PROPERTY_MTP_EXT_RESPONSE_PARAMS)
            .map(|c| collection_u32(&c))
            .unwrap_or_default();
        Ok(Container {
            code: u16::try_from(code).unwrap_or(u16::MAX),
            transaction: 0,
            params,
        })
    }
}

/// One PTP operation through WPD's MTP extension: start it, move the data phase in chunks,
/// then end it to get the camera's response.
fn execute(device: &IPortableDevice, code: u16, params: &[u32], out: Option<&[u8]>) -> Answer {
    let start = command(match out {
        Some(_) => &WPD_COMMAND_MTP_EXT_EXECUTE_COMMAND_WITH_DATA_TO_WRITE,
        None => &WPD_COMMAND_MTP_EXT_EXECUTE_COMMAND_WITH_DATA_TO_READ,
    })?;
    // SAFETY: fills the start command on a fresh collection.
    unsafe {
        start.SetUnsignedIntegerValue(&WPD_PROPERTY_MTP_EXT_OPERATION_CODE, u32::from(code))?;
        start.SetIPortableDevicePropVariantCollectionValue(
            &WPD_PROPERTY_MTP_EXT_OPERATION_PARAMS,
            &params_collection(params)?,
        )?;
        if let Some(out) = out {
            start.SetUnsignedLargeIntegerValue(
                &WPD_PROPERTY_MTP_EXT_TRANSFER_TOTAL_DATA_SIZE,
                out.len() as u64,
            )?;
        }
    }
    let started = run(device, &start)?;
    // SAFETY: reads values from the results WPD returned.
    let (context, total, chunk) = unsafe {
        (
            HSTRING::from(take_string(
                started.GetStringValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_CONTEXT)?,
            )),
            started
                .GetUnsignedLargeIntegerValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_TOTAL_DATA_SIZE)
                .unwrap_or(0),
            started
                .GetUnsignedIntegerValue(&WPD_PROPERTY_MTP_EXT_OPTIMAL_TRANSFER_BUFFER_SIZE)
                .unwrap_or(256 * 1024)
                .clamp(16 * 1024, 8 * 1024 * 1024),
        )
    };
    let moved = match out {
        Some(out) => write_data(device, &context, out, chunk).map(|()| Vec::new()),
        None => read_data(device, &context, total, chunk),
    };
    // Always end the transfer, even after a failed data phase, so the session stays usable.
    let end = command(&WPD_COMMAND_MTP_EXT_END_DATA_TRANSFER)?;
    // SAFETY: sets the context string on a fresh collection.
    unsafe { end.SetStringValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_CONTEXT, &context)? };
    let ended = run(device, &end)?;
    Ok(Reply {
        data: moved?,
        response: response(&ended)?,
    })
}

fn read_data(
    device: &IPortableDevice,
    context: &HSTRING,
    total: u64,
    chunk: u32,
) -> Result<Vec<u8>, CameraError> {
    if total > MAX_OBJECT {
        return Err(CameraError::Refused(format!(
            "the camera offers {total} bytes, over the {MAX_OBJECT}-byte limit"
        )));
    }
    let total = usize::try_from(total).unwrap_or(0);
    let mut data = Vec::with_capacity(total);
    while data.len() < total {
        let want = u32::try_from(total - data.len())
            .unwrap_or(u32::MAX)
            .min(chunk);
        let read = command(&WPD_COMMAND_MTP_EXT_READ_DATA)?;
        let buffer = vec![0u8; want as usize];
        // SAFETY: fills the read command; WPD copies `buffer`.
        unsafe {
            read.SetStringValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_CONTEXT, context)?;
            read.SetUnsignedIntegerValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_NUM_BYTES_TO_READ, want)?;
            read.SetBufferValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_DATA, &buffer)?;
        }
        let results = run(device, &read)?;
        let mut ptr: *mut u8 = std::ptr::null_mut();
        let mut len = 0u32;
        // SAFETY: WPD returns a `CoTaskMemAlloc` buffer of `len` bytes; it is copied once,
        // bounded by what was asked for, then freed.
        let got = unsafe {
            let n = results
                .GetUnsignedIntegerValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_NUM_BYTES_READ)
                .unwrap_or(0);
            results.GetBufferValue(
                &WPD_PROPERTY_MTP_EXT_TRANSFER_DATA,
                &raw mut ptr,
                &raw mut len,
            )?;
            let n = n.min(len).min(want) as usize;
            if !ptr.is_null() {
                data.extend_from_slice(std::slice::from_raw_parts(ptr, n));
                CoTaskMemFree(Some(ptr as *const _));
            }
            n
        };
        if got == 0 {
            break;
        }
    }
    Ok(data)
}

fn write_data(
    device: &IPortableDevice,
    context: &HSTRING,
    out: &[u8],
    chunk: u32,
) -> Result<(), CameraError> {
    for part in out.chunks(chunk as usize) {
        let write = command(&WPD_COMMAND_MTP_EXT_WRITE_DATA)?;
        // SAFETY: fills the write command; WPD copies `part`.
        unsafe {
            write.SetStringValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_CONTEXT, context)?;
            write.SetUnsignedIntegerValue(
                &WPD_PROPERTY_MTP_EXT_TRANSFER_NUM_BYTES_TO_WRITE,
                u32::try_from(part.len()).unwrap_or(u32::MAX),
            )?;
            write.SetBufferValue(&WPD_PROPERTY_MTP_EXT_TRANSFER_DATA, part)?;
        }
        run(device, &write)?;
    }
    Ok(())
}

use sink::EventSink;

/// The event callback, in its own module so the lints the `implement` macro's generated
/// code trips are allowed here only.
#[allow(clippy::inline_always, clippy::ref_as_ptr)]
mod sink {
    use super::{
        Arc, Event, IPortableDeviceEventCallback, IPortableDeviceEventCallback_Impl,
        IPortableDeviceValues, Instant, Mutex, Timed, WPD_EVENT_DEVICE_REMOVED,
        WPD_EVENT_PARAMETER_EVENT_ID, WPD_PROPERTY_MTP_EXT_EVENT_PARAMS, collection_u32,
        event_container, implement, vendor_event_code,
    };

    /// Receives WPD's events and keeps the PTP ones, rebuilt as containers.
    #[implement(IPortableDeviceEventCallback)]
    pub(super) struct EventSink {
        pub(super) events: Arc<Mutex<Vec<Timed>>>,
    }

    impl IPortableDeviceEventCallback_Impl for EventSink_Impl {
        fn OnEvent(
            &self,
            parameters: windows::core::Ref<IPortableDeviceValues>,
        ) -> windows::core::Result<()> {
            let Some(values) = parameters.as_ref() else {
                return Ok(());
            };
            // SAFETY: reads values from the event collection WPD passes for this call.
            let id = unsafe { values.GetGuidValue(&WPD_EVENT_PARAMETER_EVENT_ID)? };
            let event = if id == WPD_EVENT_DEVICE_REMOVED {
                Event::Removed
            } else if let Some(code) = vendor_event_code(&id) {
                // SAFETY: as above.
                let params = unsafe {
                    values
                        .GetIPortableDevicePropVariantCollectionValue(
                            &WPD_PROPERTY_MTP_EXT_EVENT_PARAMS,
                        )
                        .map(|c| collection_u32(&c))
                        .unwrap_or_default()
                };
                Event::Ptp(event_container(code, &params))
            } else {
                return Ok(());
            };
            if let Ok(mut e) = self.events.lock() {
                e.push(Timed {
                    at: Instant::now(),
                    event,
                });
            }
            Ok(())
        }
    }
}

/// WPD reports a maker's PTP event as `WPD_EVENT_MTP_VENDOR_EXTENDED_EVENTS` with the event
/// code in the high 16 bits of the GUID's first field (seen: `C2030000-5738-…`).
fn vendor_event_code(id: &GUID) -> Option<u16> {
    let base = WPD_EVENT_MTP_VENDOR_EXTENDED_EVENTS;
    (id.data2 == base.data2 && id.data3 == base.data3 && id.data4 == base.data4)
        .then(|| u16::try_from(id.data1 >> 16).ok().filter(|&c| c != 0))
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_events_carry_their_code_in_the_guid() {
        let mut id = WPD_EVENT_MTP_VENDOR_EXTENDED_EVENTS;
        // As the a6400 delivered them through WPD: C2030000-5738-4FF2-8445-BE3126691059.
        id.data1 = 0xC203_0000;
        assert_eq!(vendor_event_code(&id), Some(0xC203));
        id.data1 = 0xC201_0000;
        assert_eq!(vendor_event_code(&id), Some(0xC201));
        id.data1 = 0;
        assert_eq!(vendor_event_code(&id), None);
        assert_eq!(vendor_event_code(&WPD_EVENT_DEVICE_REMOVED), None);
    }

    #[test]
    fn a_closed_camera_refuses_to_send() {
        let mut camera = Camera {
            id: String::new(),
            name: String::new(),
            manufacturer: String::new(),
            protocol: String::new(),
            worker: None,
            events: Arc::new(Mutex::new(Vec::new())),
        };
        assert!(matches!(
            camera.send(0x1001, &[], None, Duration::from_millis(10)),
            Err(CameraError::Closed)
        ));
        assert!(camera.take_events().is_empty());
    }
}
