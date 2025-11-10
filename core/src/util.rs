// © 2025 Thor Thor
// SPDX-License-Identifier: MIT

use crate::error::CrypRqErrorCode;
use std::ffi::CStr;
use std::os::raw::c_char;

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

pub unsafe fn str_view_to_string(view: &CrypRqStrView) -> Result<String, CrypRqErrorCode> {
    if view.data.is_null() {
        return Err(CrypRqErrorCode::CRYPRQ_ERR_NULL);
    }
    let slice = std::slice::from_raw_parts(view.data as *const u8, view.len);
    Ok(std::str::from_utf8(slice)?.to_owned())
}
