// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use crate::error::CrypRqErrorCode;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Read an optional C string from a pointer.
///
/// # Safety
///
/// - If `ptr` is not null, it must point to a valid null-terminated C string
/// - The string must remain valid for the duration of the call
pub unsafe fn read_optional_cstr(ptr: *const c_char) -> Result<Option<String>, CrypRqErrorCode> {
    if ptr.is_null() {
        return Ok(None);
    }
    let value = CStr::from_ptr(ptr).to_str()?.to_owned();
    Ok(Some(value))
}

#[repr(C)]
pub struct CrypRqStrView {
    pub data: *const c_char,
    pub len: usize,
}

/// Convert a `CrypRqStrView` to a Rust `String`.
///
/// # Safety
///
/// - `view.data` must be a valid pointer to a buffer of at least `view.len` bytes
/// - The buffer must contain valid UTF-8 data
/// - The buffer must remain valid for the duration of the call
pub unsafe fn str_view_to_string(view: &CrypRqStrView) -> Result<String, CrypRqErrorCode> {
    if view.data.is_null() {
        return Err(CrypRqErrorCode::CRYPRQ_ERR_NULL);
    }
    let slice = std::slice::from_raw_parts(view.data as *const u8, view.len);
    Ok(std::str::from_utf8(slice)?.to_owned())
}
