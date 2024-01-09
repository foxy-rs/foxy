use std::sync::{Mutex, MutexGuard};
use windows::{
    core::{ComInterface, PCSTR},
    Win32::{
        Foundation::LPARAM,
        Graphics::{
            Direct3D12::{
                D3D12GetDebugInterface, ID3D12Debug6, ID3D12Device12, ID3D12InfoQueue1,
                D3D12_MESSAGE_CALLBACK_FLAG_NONE, D3D12_MESSAGE_CATEGORY, D3D12_MESSAGE_ID,
                D3D12_MESSAGE_SEVERITY,
            },
            Dxgi::{
                DXGIGetDebugInterface1, IDXGIDebug1, IDXGIInfoQueue, DXGI_DEBUG_ALL,
                DXGI_DEBUG_RLO_DETAIL, DXGI_DEBUG_RLO_IGNORE_INTERNAL,
            },
        },
    },
};

extern "system" fn d3d12_debug_callback(
    category: D3D12_MESSAGE_CATEGORY,
    severity: D3D12_MESSAGE_SEVERITY,
    id: D3D12_MESSAGE_ID,
    description: PCSTR,
    _context: *mut core::ffi::c_void,
) {
    let message = unsafe { std::ffi::CStr::from_ptr(description.0 as *const i8) }
        .to_string_lossy()
        .into_owned();

    // Use the `tracing` crate to log the message
    tracing::debug!(
        "D3D12 Debug Message: Category={:?} Severity={:?} ID={:?} Description={}",
        category,
        severity,
        id,
        message
    );
}

// extern "system" fn dxgi_debug_callback(
//     _message: *const DXGI_INFO_QUEUE_MESSAGE,
//     _context: *mut core::ffi::c_void,
// ) {
//     let category = unsafe { (*_message).Category };
//     let severity = unsafe { (*_message).Severity };
//     let id = unsafe { (*_message).ID };
//     let message = unsafe { std::ffi::CStr::from_ptr((*_message).pDescription as *const i8) }
//         .to_string_lossy()
//         .into_owned();
//     // Convert the message to a Rust string and log it using `tracing`
//     // Note: You'll need to adjust the code to match the actual parameters and types
//     // provided by the DXGI debug callback.
//     tracing::debug!(
//         "DXGI Debug Message: Category={:?} Severity={:?} ID={:?} Description={}",
//         category,
//         severity,
//         id,
//         message
//     );
// }

static VALIDATION_LAYER_INSTANCE: Mutex<ValidationLayer> = Mutex::new(ValidationLayer {
    d3d12_debug: None,
    dxgi_debug: None,
    d3d12_info_queue: None,
    dxgi_info_queue: None,
    d3d12_info_queue_callback_cookie: LPARAM(0),
});

pub struct ValidationLayer {
    d3d12_debug: Option<ID3D12Debug6>,
    dxgi_debug: Option<IDXGIDebug1>,
    d3d12_info_queue: Option<ID3D12InfoQueue1>,
    dxgi_info_queue: Option<IDXGIInfoQueue>,
    d3d12_info_queue_callback_cookie: LPARAM,
}

impl ValidationLayer {
    pub fn instance() -> MutexGuard<'static, Self> {
        VALIDATION_LAYER_INSTANCE.lock().unwrap()
    }

    pub fn init(&mut self) -> bool {
        if cfg!(debug_assertions) {
            // init d3d12 debug layer
            unsafe {
                if D3D12GetDebugInterface(std::ptr::addr_of_mut!(self.d3d12_debug)).is_ok() {
                    self.d3d12_debug.as_mut().unwrap().EnableDebugLayer();

                    if let Ok(dxgi_debug) = DXGIGetDebugInterface1(0) {
                        self.dxgi_debug = Some(dxgi_debug);
                        self.dxgi_debug
                            .as_mut()
                            .unwrap()
                            .EnableLeakTrackingForThread();

                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn register_callbacks(&mut self, device: ID3D12Device12) {
        match device.cast::<ID3D12InfoQueue1>() {
            Ok(d3d12_info_queue) => {
                self.d3d12_info_queue = Some(d3d12_info_queue);
            }
            Err(error) => tracing::error!("{error}"),
        }

        match device.cast::<IDXGIInfoQueue>() {
            Ok(dxgi_info_queue) => {
                self.dxgi_info_queue = Some(dxgi_info_queue);
            }
            Err(error) => tracing::error!("{error}"),
        }

        if let Some(d3d12_info_queue) = self.d3d12_info_queue.as_ref() {
            unsafe {
                let mut d3d12_info_queue_callback_cookie: LPARAM = LPARAM(0);
                if let Err(error) = d3d12_info_queue.RegisterMessageCallback(
                    Some(d3d12_debug_callback),
                    D3D12_MESSAGE_CALLBACK_FLAG_NONE, // or D3D12_MESSAGE_CALLBACK_IGNORE_FILTERS
                    std::ptr::null_mut(),
                    &mut d3d12_info_queue_callback_cookie as *mut LPARAM as *mut u32,
                ) {
                    tracing::error!("{error}");
                }
            }
        }
    }

    pub fn shutdown(&mut self) {
        if cfg!(debug_assertions) {
            if let Some(dxgi_debug) = &self.dxgi_debug {
                unsafe {
                    if let Err(error) = dxgi_debug.ReportLiveObjects(
                        DXGI_DEBUG_ALL,
                        DXGI_DEBUG_RLO_DETAIL | DXGI_DEBUG_RLO_IGNORE_INTERNAL,
                    ) {
                        tracing::error!("{error}");
                    }
                }
            }

            if let Some(info_queue) = &self.d3d12_info_queue {
                unsafe {
                    if let Err(error) = info_queue
                        .UnregisterMessageCallback(self.d3d12_info_queue_callback_cookie.0 as u32)
                    {
                        tracing::error!("{error}");
                    }
                }
            }
        }
    }
}
