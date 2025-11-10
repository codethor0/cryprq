// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! CrypRQ core FFI layer.
//!
//! Exposes a stable C ABI for host platforms (Android, iOS, macOS, Windows).

mod error;
mod ffi;
mod handle;
mod util;

pub use error::CrypRqErrorCode;
pub use ffi::*;
pub use util::CrypRqStrView;
