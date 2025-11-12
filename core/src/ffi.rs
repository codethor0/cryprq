// © 2025 Thor Thor
// SPDX-License-Identifier: MIT

use crate::error::CrypRqErrorCode;
use crate::handle::CrypRqHandle;
use crate::util::{read_optional_cstr, str_view_to_string, CrypRqStrView};
use std::ffi::CStr;
use std::os::raw::c_char;

#[repr(C)]
pub struct CrypRqConfig {
    pub log_level: *const c_char,
    pub allow_peers: *const CrypRqStrView,
    pub allow_peers_len: usize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum CrypRqConnectionMode {
    CRYPRQ_CONNECTION_MODE_LISTEN = 0,
    CRYPRQ_CONNECTION_MODE_DIAL = 1,
}

#[repr(C)]
pub struct CrypRqPeerParams {
    pub mode: CrypRqConnectionMode,
    pub multiaddr: *const c_char,
}

/// Initialize CrypRQ with the given configuration.
///
/// # Safety
///
/// - `config` must be a valid pointer to a `CrypRqConfig` or null
/// - `out_handle` must be a valid pointer to a `*mut CrypRqHandle` or null
/// - If `config` is not null, it must point to valid memory for the duration of the call
/// - If `config.allow_peers` is not null, it must point to an array of `allow_peers_len` valid `CrypRqStrView` structs
/// - If `config.multiaddr` is not null, it must point to a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn cryprq_init(
    config: *const CrypRqConfig,
    out_handle: *mut *mut CrypRqHandle,
) -> CrypRqErrorCode {
    if config.is_null() || out_handle.is_null() {
        return CrypRqErrorCode::CRYPRQ_ERR_NULL;
    }

    let cfg = &*config;
    match init_inner(cfg) {
        Ok(handle) => {
            *out_handle = Box::into_raw(Box::new(handle));
            CrypRqErrorCode::CRYPRQ_OK
        }
        Err(code) => code,
    }
}

/// Inner initialization function.
///
/// # Safety
///
/// - `cfg` must be a valid reference to a `CrypRqConfig`
/// - If `cfg.allow_peers` is not null, it must point to an array of `cfg.allow_peers_len` valid `CrypRqStrView` structs
/// - If `cfg.multiaddr` is not null, it must point to a valid null-terminated C string
unsafe fn init_inner(cfg: &CrypRqConfig) -> Result<CrypRqHandle, CrypRqErrorCode> {
    if cfg.allow_peers_len > 0 && cfg.allow_peers.is_null() {
        return Err(CrypRqErrorCode::CRYPRQ_ERR_NULL);
    }

    if let Some(level) = read_optional_cstr(cfg.log_level)? {
        std::env::set_var("RUST_LOG", level);
    }

    let allow = if cfg.allow_peers_len > 0 {
        std::slice::from_raw_parts(cfg.allow_peers, cfg.allow_peers_len)
            .iter()
            .map(|view| unsafe { str_view_to_string(view) })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    CrypRqHandle::new(allow)
}

/// Connect to a peer using the given parameters.
///
/// # Safety
///
/// - `handle` must be a valid pointer to a `CrypRqHandle` created by `cryprq_init`
/// - `params` must be a valid pointer to a `CrypRqPeerParams` or null
/// - If `params` is not null, it must point to valid memory for the duration of the call
/// - If `params.multiaddr` is not null, it must point to a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn cryprq_connect(
    handle: *mut CrypRqHandle,
    params: *const CrypRqPeerParams,
) -> CrypRqErrorCode {
    if handle.is_null() || params.is_null() {
        return CrypRqErrorCode::CRYPRQ_ERR_NULL;
    }
    let handle = &*handle;
    let params = &*params;

    let addr = if params.multiaddr.is_null() {
        return CrypRqErrorCode::CRYPRQ_ERR_NULL;
    } else {
        match CStr::from_ptr(params.multiaddr).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => return CrypRqErrorCode::CRYPRQ_ERR_UTF8,
        }
    };

    let result = match params.mode {
        CrypRqConnectionMode::CRYPRQ_CONNECTION_MODE_LISTEN => handle.spawn_listener(addr),
        CrypRqConnectionMode::CRYPRQ_CONNECTION_MODE_DIAL => handle.spawn_dialer(addr),
    };

    match result {
        Ok(()) => CrypRqErrorCode::CRYPRQ_OK,
        Err(code) => code,
    }
}

/// Read a packet from the connection.
///
/// # Safety
///
/// - `handle` must be a valid pointer to a `CrypRqHandle`
/// - `buffer` must be a valid pointer to a buffer of at least `len` bytes
/// - `out_len` must be a valid pointer to a `usize` or null
/// - The buffer must remain valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn cryprq_read_packet(
    _handle: *mut CrypRqHandle,
    _buffer: *mut u8,
    _len: usize,
    _out_len: *mut usize,
) -> CrypRqErrorCode {
    CrypRqErrorCode::CRYPRQ_ERR_UNSUPPORTED
}

/// Write a packet to the connection.
///
/// # Safety
///
/// - `handle` must be a valid pointer to a `CrypRqHandle`
/// - `buffer` must be a valid pointer to a buffer of at least `len` bytes
/// - The buffer must remain valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn cryprq_write_packet(
    _handle: *mut CrypRqHandle,
    _buffer: *const u8,
    _len: usize,
) -> CrypRqErrorCode {
    CrypRqErrorCode::CRYPRQ_ERR_UNSUPPORTED
}

/// Notify CrypRQ of a network change event.
///
/// # Safety
///
/// - `handle` must be a valid pointer to a `CrypRqHandle` created by `cryprq_init`
#[no_mangle]
pub unsafe extern "C" fn cryprq_on_network_change(_handle: *mut CrypRqHandle) -> CrypRqErrorCode {
    CrypRqErrorCode::CRYPRQ_OK
}

/// Close and deallocate a CrypRQ handle.
///
/// # Safety
///
/// - `handle` must be a valid pointer to a `CrypRqHandle` created by `cryprq_init`
/// - After this call, `handle` is invalid and must not be used
/// - This function takes ownership of the handle and deallocates it
#[no_mangle]
pub unsafe extern "C" fn cryprq_close(handle: *mut CrypRqHandle) -> CrypRqErrorCode {
    if handle.is_null() {
        return CrypRqErrorCode::CRYPRQ_ERR_NULL;
    }
    drop(Box::from_raw(handle));
    CrypRqErrorCode::CRYPRQ_OK
}
