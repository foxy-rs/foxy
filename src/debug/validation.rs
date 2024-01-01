use std::sync::{Mutex, MutexGuard};
use tracing::error;
use windows::Win32::Graphics::{
    Direct3D12::{D3D12GetDebugInterface, ID3D12Debug6},
    Dxgi::{
        DXGIGetDebugInterface1, IDXGIDebug1, DXGI_DEBUG_ALL, DXGI_DEBUG_RLO_DETAIL,
        DXGI_DEBUG_RLO_IGNORE_INTERNAL,
    },
};

static VALIDATION_LAYER_INSTANCE: Mutex<ValidationLayer> = Mutex::new(ValidationLayer {
    d3d12_debug: None,
    dxgi_debug: None,
});

pub struct ValidationLayer {
    d3d12_debug: Option<ID3D12Debug6>,
    dxgi_debug: Option<IDXGIDebug1>,
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

    pub fn shutdown(&mut self) {
        if cfg!(debug_assertions) {
            if let Some(dxgi_debug) = &self.dxgi_debug {
                unsafe {
                    if let Err(error) = dxgi_debug.ReportLiveObjects(
                        DXGI_DEBUG_ALL,
                        DXGI_DEBUG_RLO_DETAIL | DXGI_DEBUG_RLO_IGNORE_INTERNAL,
                    ) {
                        error!("{error}");
                    }
                }
            }
        }
    }
}
