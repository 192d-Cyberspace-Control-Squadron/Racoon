use crate::bindings::*;
use crate::status::SaiStatus;
use libloading::{Library, Symbol};
use racoon_common::{RacoonError, Result};
use std::ffi::CString;
use std::os::raw::c_void;
use std::sync::Arc;
use tracing::{info, warn};

type SaiApiQueryFn =
    unsafe extern "C" fn(api: sai_api_t, api_method_table: *mut *const c_void) -> sai_status_t;

type SaiApiInitializeFn = unsafe extern "C" fn(
    flags: u64,
    service_method_table: *const sai_service_method_table_t,
) -> sai_status_t;

type SaiApiUninitializeFn = unsafe extern "C" fn() -> sai_status_t;

/// SAI Adapter - manages dynamic loading and interaction with vendor SAI libraries
pub struct SaiAdapter {
    _library: Library,
    api_query: Symbol<'static, SaiApiQueryFn>,
    api_uninitialize: Symbol<'static, SaiApiUninitializeFn>,

    // Cached API table pointers
    switch_api: *const sai_switch_api_t,
    port_api: *const sai_port_api_t,
    vlan_api: *const sai_vlan_api_t,
    fdb_api: *const sai_fdb_api_t,
    lag_api: *const sai_lag_api_t,
    bridge_api: *const sai_bridge_api_t,
}

unsafe impl Send for SaiAdapter {}
unsafe impl Sync for SaiAdapter {}

impl SaiAdapter {
    /// Load a SAI library from the specified path
    pub fn load(library_path: &str) -> Result<Arc<Self>> {
        info!("Loading SAI library from: {}", library_path);

        // Load the shared library
        let library = unsafe {
            Library::new(library_path).map_err(|e| {
                RacoonError::LibraryLoad(format!("Failed to load SAI library: {}", e))
            })?
        };

        // Get sai_api_query function
        let api_query: Symbol<SaiApiQueryFn> = unsafe {
            library.get(b"sai_api_query\0").map_err(|e| {
                RacoonError::LibraryLoad(format!("Failed to find sai_api_query: {}", e))
            })?
        };

        // Get sai_api_initialize function
        let api_initialize: Symbol<SaiApiInitializeFn> = unsafe {
            library.get(b"sai_api_initialize\0").map_err(|e| {
                RacoonError::LibraryLoad(format!("Failed to find sai_api_initialize: {}", e))
            })?
        };

        // Get sai_api_uninitialize function
        let api_uninitialize: Symbol<SaiApiUninitializeFn> = unsafe {
            library.get(b"sai_api_uninitialize\0").map_err(|e| {
                RacoonError::LibraryLoad(format!("Failed to find sai_api_uninitialize: {}", e))
            })?
        };

        // Initialize SAI
        let status = unsafe {
            let service_table: sai_service_method_table_t = std::mem::zeroed();
            api_initialize(0, &service_table)
        };

        SaiStatus::from(status).to_result()?;
        info!("SAI library initialized successfully");

        // Query all API tables
        let switch_api = Self::query_api(&api_query, sai_api_t_SAI_API_SWITCH)?;
        let port_api = Self::query_api(&api_query, sai_api_t_SAI_API_PORT)?;
        let vlan_api = Self::query_api(&api_query, sai_api_t_SAI_API_VLAN)?;
        let fdb_api = Self::query_api(&api_query, sai_api_t_SAI_API_FDB)?;
        let lag_api = Self::query_api(&api_query, sai_api_t_SAI_API_LAG)?;
        let bridge_api = Self::query_api(&api_query, sai_api_t_SAI_API_BRIDGE)?;

        // Leak the symbols to get 'static lifetime
        let api_query = unsafe { std::mem::transmute(api_query) };
        let api_uninitialize = unsafe { std::mem::transmute(api_uninitialize) };

        Ok(Arc::new(Self {
            _library: library,
            api_query,
            api_uninitialize,
            switch_api,
            port_api,
            vlan_api,
            fdb_api,
            lag_api,
            bridge_api,
        }))
    }

    /// Query a specific SAI API table
    fn query_api<T>(api_query: &Symbol<SaiApiQueryFn>, api_type: sai_api_t) -> Result<*const T> {
        let mut api_ptr: *const c_void = std::ptr::null();

        let status = unsafe { api_query(api_type, &mut api_ptr as *mut *const c_void) };

        SaiStatus::from(status).to_result()?;

        if api_ptr.is_null() {
            return Err(RacoonError::Sai("API table pointer is null".to_string()));
        }

        Ok(api_ptr as *const T)
    }

    /// Get the Switch API table
    pub fn get_switch_api(&self) -> &sai_switch_api_t {
        unsafe { &*self.switch_api }
    }

    /// Get the Port API table
    pub fn get_port_api(&self) -> &sai_port_api_t {
        unsafe { &*self.port_api }
    }

    /// Get the VLAN API table
    pub fn get_vlan_api(&self) -> &sai_vlan_api_t {
        unsafe { &*self.vlan_api }
    }

    /// Get the FDB API table
    pub fn get_fdb_api(&self) -> &sai_fdb_api_t {
        unsafe { &*self.fdb_api }
    }

    /// Get the LAG API table
    pub fn get_lag_api(&self) -> &sai_lag_api_t {
        unsafe { &*self.lag_api }
    }

    /// Get the Bridge API table
    pub fn get_bridge_api(&self) -> &sai_bridge_api_t {
        unsafe { &*self.bridge_api }
    }
}

impl Drop for SaiAdapter {
    fn drop(&mut self) {
        warn!("Uninitializing SAI library");
        unsafe {
            let status = (self.api_uninitialize)();
            if SaiStatus::from(status).is_error() {
                warn!("Failed to uninitialize SAI: {:?}", SaiStatus::from(status));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run when SAI library is available
    fn test_load_sai_library() {
        let result = SaiAdapter::load("/usr/lib/libsai.so");
        // Will fail if no SAI library present, which is expected in CI
        if result.is_ok() {
            println!("SAI library loaded successfully");
        }
    }
}
