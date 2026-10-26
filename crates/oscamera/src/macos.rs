// SPDX-License-Identifier: GPL-3.0-only
//! The macOS transport: `ImageCaptureCore` finds cameras, opens a session and carries any PTP
//! operation, standard or a maker's, with `requestSendPTPCommand` (ADR-0026). No driver is
//! installed and `ptpcamerad` keeps owning the USB device.
//!
//! Everything runs on the main thread: `ImageCaptureCore` reports to delegates there, and a
//! call that waits for an answer spins the run loop until the answer arrives or time runs out,
//! so no Objective-C object crosses threads.

use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send};
use objc2_foundation::{NSArray, NSData, NSDate, NSError, NSObject, NSObjectProtocol, NSRunLoop};
use objc2_image_capture_core::{
    ICCameraDevice, ICCameraDeviceDelegate, ICCameraItem, ICDevice, ICDeviceBrowser,
    ICDeviceBrowserDelegate, ICDeviceDelegate, ICDeviceTypeMask,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CameraError {
    #[error("this must run on the main thread")]
    NotMainThread,
    #[error("the camera did not answer within {0:?}")]
    Timeout(Duration),
    #[error("the camera refused: {0}")]
    Refused(String),
    #[error("the camera's answer is not PTP: {0}")]
    Ptp(#[from] crate::ptp::PtpError),
}

/// Something the camera reported while a session is open.
#[derive(Debug, Clone)]
pub enum Event {
    /// New files on the camera (a shot), with their names and PTP handles.
    ItemsAdded(Vec<(String, u32)>),
    /// A raw PTP event container.
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

/// Spins the main run loop in short slices until `done` returns a value or time runs out.
fn spin<T>(timeout: Duration, mut done: impl FnMut() -> Option<T>) -> Result<T, CameraError> {
    let start = Instant::now();
    let run_loop = NSRunLoop::currentRunLoop();
    loop {
        if let Some(v) = done() {
            return Ok(v);
        }
        if start.elapsed() >= timeout {
            return Err(CameraError::Timeout(timeout));
        }
        let until = NSDate::dateWithTimeIntervalSinceNow(0.02);
        run_loop.runUntilDate(&until);
    }
}

/// Runs the main run loop for a while, so delegates and events are delivered.
pub fn pump(duration: Duration) {
    let until = NSDate::dateWithTimeIntervalSinceNow(duration.as_secs_f64());
    NSRunLoop::currentRunLoop().runUntilDate(&until);
}

struct BrowserState {
    cameras: RefCell<Vec<Retained<ICCameraDevice>>>,
    enumerated: RefCell<bool>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "FotollectionCameraBrowser"]
    #[ivars = BrowserState]
    struct BrowserDelegate;

    unsafe impl NSObjectProtocol for BrowserDelegate {}

    unsafe impl ICDeviceBrowserDelegate for BrowserDelegate {
        #[unsafe(method(deviceBrowser:didAddDevice:moreComing:))]
        fn did_add(&self, _browser: &ICDeviceBrowser, device: &ICDevice, more: bool) {
            // SAFETY: `device` is a live ICDevice; reading its type has no side effect.
            let is_camera = unsafe { device.r#type() }.0 & ICDeviceTypeMask::Camera.0 != 0;
            if is_camera {
                let device: Retained<ICDevice> = device.retain();
                if let Ok(camera) = device.downcast::<ICCameraDevice>() {
                    self.ivars().cameras.borrow_mut().push(camera);
                }
            }
            if !more {
                *self.ivars().enumerated.borrow_mut() = true;
            }
        }

        #[unsafe(method(deviceBrowser:didRemoveDevice:moreGoing:))]
        fn did_remove(&self, _browser: &ICDeviceBrowser, device: &ICDevice, _more: bool) {
            let gone: *const ICDevice = device;
            self.ivars()
                .cameras
                .borrow_mut()
                .retain(|c| !std::ptr::eq(Retained::as_ptr(c).cast::<ICDevice>(), gone));
        }

        #[unsafe(method(deviceBrowserDidEnumerateLocalDevices:))]
        fn did_enumerate(&self, _browser: &ICDeviceBrowser) {
            *self.ivars().enumerated.borrow_mut() = true;
        }
    }
);

impl BrowserDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(BrowserState {
            cameras: RefCell::new(Vec::new()),
            enumerated: RefCell::new(false),
        });
        // SAFETY: NSObject's `init` on a freshly allocated object.
        unsafe { msg_send![super(this), init] }
    }
}

type Events = Rc<RefCell<Vec<Timed>>>;

struct CameraState {
    events: Events,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "FotollectionCameraDelegate"]
    #[ivars = CameraState]
    struct CameraDelegate;

    unsafe impl NSObjectProtocol for CameraDelegate {}

    unsafe impl ICDeviceDelegate for CameraDelegate {
        #[unsafe(method(didRemoveDevice:))]
        fn did_remove_device(&self, _device: &ICDevice) {
            self.push(Event::Removed);
        }

        #[unsafe(method(device:didOpenSessionWithError:))]
        fn did_open(&self, _device: &ICDevice, _error: Option<&NSError>) {}

        #[unsafe(method(device:didCloseSessionWithError:))]
        fn did_close(&self, _device: &ICDevice, _error: Option<&NSError>) {}
    }

    unsafe impl ICCameraDeviceDelegate for CameraDelegate {
        #[unsafe(method(cameraDevice:didAddItems:))]
        fn did_add_items(&self, _camera: &ICCameraDevice, items: &NSArray<ICCameraItem>) {
            let added = items
                .iter()
                .map(|item| {
                    // SAFETY: plain property reads on a live ICCameraItem.
                    let name = unsafe { item.name() }
                        .map(|n| n.to_string())
                        .unwrap_or_default();
                    // SAFETY: as above.
                    let handle = unsafe { item.ptpObjectHandle() };
                    (name, handle)
                })
                .collect();
            self.push(Event::ItemsAdded(added));
        }

        #[unsafe(method(cameraDevice:didRemoveItems:))]
        fn did_remove_items(&self, _camera: &ICCameraDevice, _items: &NSArray<ICCameraItem>) {}

        #[unsafe(method(cameraDevice:didReceiveThumbnail:forItem:error:))]
        fn did_thumbnail(
            &self,
            _camera: &ICCameraDevice,
            _thumbnail: Option<&objc2_core_graphics::CGImage>,
            _item: &ICCameraItem,
            _error: Option<&NSError>,
        ) {
        }

        #[unsafe(method(cameraDevice:didReceiveMetadata:forItem:error:))]
        fn did_metadata(
            &self,
            _camera: &ICCameraDevice,
            _metadata: Option<&objc2_foundation::NSDictionary>,
            _item: &ICCameraItem,
            _error: Option<&NSError>,
        ) {
        }

        #[unsafe(method(cameraDevice:didRenameItems:))]
        fn did_rename(&self, _camera: &ICCameraDevice, _items: &NSArray<ICCameraItem>) {}

        #[unsafe(method(cameraDeviceDidChangeCapability:))]
        fn did_change_capability(&self, _camera: &ICCameraDevice) {}

        #[unsafe(method(cameraDevice:didReceivePTPEvent:))]
        fn did_ptp_event(&self, _camera: &ICCameraDevice, data: &NSData) {
            self.push(Event::Ptp(data.to_vec()));
        }

        #[unsafe(method(deviceDidBecomeReadyWithCompleteContentCatalog:))]
        fn did_become_ready(&self, _device: &ICCameraDevice) {}

        #[unsafe(method(cameraDeviceDidRemoveAccessRestriction:))]
        fn did_remove_restriction(&self, _device: &ICDevice) {}

        #[unsafe(method(cameraDeviceDidEnableAccessRestriction:))]
        fn did_enable_restriction(&self, _device: &ICDevice) {}
    }
);

impl CameraDelegate {
    fn new(mtm: MainThreadMarker, events: Events) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(CameraState { events });
        // SAFETY: NSObject's `init` on a freshly allocated object.
        unsafe { msg_send![super(this), init] }
    }

    fn push(&self, event: Event) {
        self.ivars().events.borrow_mut().push(Timed {
            at: Instant::now(),
            event,
        });
    }
}

/// Looks for cameras connected to this Mac.
pub struct Browser {
    browser: Retained<ICDeviceBrowser>,
    delegate: Retained<BrowserDelegate>,
}

impl Browser {
    /// Starts looking; cameras arrive while the run loop runs.
    pub fn start() -> Result<Self, CameraError> {
        let mtm = MainThreadMarker::new().ok_or(CameraError::NotMainThread)?;
        let delegate = BrowserDelegate::new(mtm);
        // SAFETY: a new browser, configured before `start`; the delegate outlives it (kept in
        // `Self`, which stops the browser on drop).
        let browser = unsafe {
            let browser = ICDeviceBrowser::new();
            browser.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            browser.setBrowsedDeviceTypeMask(ICDeviceTypeMask(
                ICDeviceTypeMask::Camera.0 | 0x0000_FF00,
            ));
            browser.start();
            browser
        };
        Ok(Self { browser, delegate })
    }

    /// The cameras found so far, after waiting up to `timeout` for the first enumeration.
    pub fn cameras(&self, timeout: Duration) -> Vec<Camera> {
        let ivars = self.delegate.ivars();
        let _ = spin(timeout, || {
            (*ivars.enumerated.borrow() && !ivars.cameras.borrow().is_empty()).then_some(())
        });
        ivars
            .cameras
            .borrow()
            .iter()
            .map(|device| Camera {
                device: device.clone(),
                delegate: None,
                events: Rc::new(RefCell::new(Vec::new())),
                transaction: 1,
            })
            .collect()
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        // SAFETY: stopping a started browser.
        unsafe { self.browser.stop() };
    }
}

/// A camera found by the browser.
pub struct Camera {
    device: Retained<ICCameraDevice>,
    delegate: Option<Retained<CameraDelegate>>,
    events: Events,
    transaction: u32,
}

/// What a PTP operation brought back.
#[derive(Debug, Clone)]
pub struct Reply {
    /// The data phase, without a container header.
    pub data: Vec<u8>,
    pub response: crate::ptp::Container,
}

impl Camera {
    pub fn name(&self) -> String {
        // SAFETY: plain property read.
        unsafe { self.device.name() }
            .map(|n| n.to_string())
            .unwrap_or_default()
    }

    pub fn product_kind(&self) -> String {
        // SAFETY: plain property read.
        unsafe { self.device.productKind() }
            .map(|n| n.to_string())
            .unwrap_or_default()
    }

    /// `ImageCaptureCore`'s capability strings (for example `ICCameraDeviceCanTakePicture`).
    pub fn capabilities(&self) -> Vec<String> {
        // SAFETY: plain property read.
        unsafe { self.device.capabilities() }
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Opens a session and starts collecting events.
    pub fn open(&mut self, timeout: Duration) -> Result<(), CameraError> {
        let mtm = MainThreadMarker::new().ok_or(CameraError::NotMainThread)?;
        let delegate = CameraDelegate::new(mtm, self.events.clone());
        // SAFETY: the delegate is kept in `self` for as long as the device may call it.
        unsafe {
            self.device
                .setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        }
        self.delegate = Some(delegate);
        let result: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        let slot = result.clone();
        let block = RcBlock::new(move |error: *mut NSError| {
            // SAFETY: ImageCaptureCore passes null or a valid NSError for the call's duration.
            let outcome = unsafe { error.as_ref() }
                .map_or(Ok(()), |e| Err(e.localizedDescription().to_string()));
            if let Ok(mut s) = slot.lock() {
                *s = Some(outcome);
            }
        });
        // SAFETY: a valid block; no options.
        unsafe {
            self.device
                .requestOpenSessionWithOptions_completion(None, &block);
        }
        let outcome = spin(timeout, || result.lock().ok().and_then(|mut s| s.take()))?;
        outcome.map_err(CameraError::Refused)
    }

    /// `ImageCaptureCore`'s own "take a picture" (standard `InitiateCapture` underneath).
    pub fn take_picture(&self) {
        // SAFETY: a request on an open session.
        unsafe { self.device.requestTakePicture() };
    }

    /// Sends one PTP operation, with an optional data phase from the computer, and waits.
    pub fn send(
        &mut self,
        code: u16,
        params: &[u32],
        out: Option<&[u8]>,
        timeout: Duration,
    ) -> Result<Reply, CameraError> {
        type Answer = Result<(Vec<u8>, Vec<u8>), String>;
        let command = NSData::with_bytes(&crate::ptp::command(code, self.transaction, params));
        self.transaction = self.transaction.wrapping_add(1).max(1);
        let out = out.map(NSData::with_bytes);
        let result: Arc<Mutex<Option<Answer>>> = Arc::new(Mutex::new(None));
        let slot = result.clone();
        let block = RcBlock::new(
            move |data: NonNull<NSData>, response: NonNull<NSData>, error: *mut NSError| {
                // SAFETY: ImageCaptureCore passes valid objects for the call's duration.
                let outcome = match unsafe { error.as_ref() } {
                    Some(e) => Err(e.localizedDescription().to_string()),
                    None => Ok(unsafe { (data.as_ref().to_vec(), response.as_ref().to_vec()) }),
                };
                if let Ok(mut s) = slot.lock() {
                    *s = Some(outcome);
                }
            },
        );
        // SAFETY: valid command and data objects and a valid block; the camera is in session.
        unsafe {
            self.device
                .requestSendPTPCommand_outData_completion(&command, out.as_deref(), &block);
        }
        let (data, response) = spin(timeout, || result.lock().ok().and_then(|mut s| s.take()))?
            .map_err(CameraError::Refused)?;
        Ok(Reply {
            data: crate::ptp::payload(&data).to_vec(),
            response: crate::ptp::response(&response)?,
        })
    }

    /// The events received since the last call.
    pub fn take_events(&self) -> Vec<Timed> {
        std::mem::take(&mut *self.events.borrow_mut())
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        if self.delegate.is_some() {
            // SAFETY: closing the session this value opened, then detaching the delegate.
            unsafe {
                self.device.requestCloseSession();
                self.device.setDelegate(None);
            }
        }
    }
}
