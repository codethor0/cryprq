// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

mod error;
mod ffi;
mod handle;
mod record;
mod util;

pub use error::CrypRqErrorCode;
pub use ffi::*;
pub use record::{
    Record, RecordHeader, MSG_TYPE_CONTROL, MSG_TYPE_DATA, MSG_TYPE_FILE_ACK, MSG_TYPE_FILE_CHUNK,
    MSG_TYPE_FILE_META, MSG_TYPE_VPN_PACKET, PROTOCOL_VERSION, RECORD_HEADER_SIZE,
};
pub use util::CrypRqStrView;
