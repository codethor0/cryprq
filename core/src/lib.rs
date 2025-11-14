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
mod record;
mod util;

pub use error::CrypRqErrorCode;
pub use ffi::*;
pub use record::{
    Record, RecordHeader, MSG_TYPE_CONTROL, MSG_TYPE_DATA, MSG_TYPE_FILE_ACK,
    MSG_TYPE_FILE_CHUNK, MSG_TYPE_FILE_META, MSG_TYPE_VPN_PACKET, PROTOCOL_VERSION,
    RECORD_HEADER_SIZE,
};
pub use util::CrypRqStrView;
