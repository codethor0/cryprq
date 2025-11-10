// © 2025 Thor Thor
// SPDX-License-Identifier: MIT

use std::ffi::NulError;
use std::str::Utf8Error;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CrypRqErrorCode {
    CRYPRQ_OK = 0,
    CRYPRQ_ERR_NULL = 1,
    CRYPRQ_ERR_UTF8 = 2,
    CRYPRQ_ERR_INVALID_ARGUMENT = 3,
    CRYPRQ_ERR_ALREADY_CONNECTED = 4,
    CRYPRQ_ERR_UNSUPPORTED = 5,
    CRYPRQ_ERR_RUNTIME = 6,
    CRYPRQ_ERR_INTERNAL = 255,
}

impl CrypRqErrorCode {
    pub fn is_ok(self) -> bool {
        matches!(self, CrypRqErrorCode::CRYPRQ_OK)
    }
}

impl From<anyhow::Error> for CrypRqErrorCode {
    fn from(_: anyhow::Error) -> Self {
        CrypRqErrorCode::CRYPRQ_ERR_INTERNAL
    }
}

impl From<Utf8Error> for CrypRqErrorCode {
    fn from(_: Utf8Error) -> Self {
        CrypRqErrorCode::CRYPRQ_ERR_UTF8
    }
}

impl From<NulError> for CrypRqErrorCode {
    fn from(_: NulError) -> Self {
        CrypRqErrorCode::CRYPRQ_ERR_INVALID_ARGUMENT
    }
}
