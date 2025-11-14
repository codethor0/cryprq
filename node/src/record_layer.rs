// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! CrypRQ record layer send/receive functions
//!
//! Implements the record boundary for sending and receiving CrypRQ records
//! as specified in cryp-rq-protocol-v1.md Section 6

use cryprq_core::{Record, RecordHeader, PROTOCOL_VERSION};
use std::io;
use zeroize::Zeroize;

use crate::Epoch;

/// Directional keys for encryption/decryption
#[derive(Clone, Debug)]
pub struct DirectionKeys {
    pub key: [u8; 32],
    pub iv: [u8; 12],
}

impl zeroize::Zeroize for DirectionKeys {
    fn zeroize(&mut self) {
        self.key.zeroize();
        self.iv.zeroize();
    }
}

impl Drop for DirectionKeys {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Sends a CrypRQ record over the transport
///
/// As specified in Section 6.1-6.2:
/// - Constructs record header
/// - Encrypts plaintext with AEAD using TLS 1.3-style nonce
/// - Encodes to bytes
///
/// # Arguments
///
/// * `epoch` - Current epoch (u8)
/// * `stream_id` - Stream identifier
/// * `sequence_number` - Sequence number for this direction/stream
/// * `message_type` - Message type (MSG_TYPE_DATA, MSG_TYPE_VPN_PACKET, etc.)
/// * `flags` - Message flags
/// * `plaintext` - Plaintext payload to encrypt
/// * `keys` - Directional keys (key + IV)
///
/// # Returns
///
/// Encoded record bytes ready to send over transport
pub fn send_record(
    epoch: Epoch,
    stream_id: u32,
    sequence_number: u64,
    message_type: u8,
    flags: u8,
    plaintext: &[u8],
    keys: &DirectionKeys,
) -> io::Result<Vec<u8>> {
    // Construct and encrypt record
    let record = Record::encrypt(
        PROTOCOL_VERSION,
        message_type,
        flags,
        epoch.value(),
        stream_id,
        sequence_number,
        plaintext,
        &keys.key,
        &keys.iv,
    )?;

    // Encode to bytes
    Ok(record.to_bytes())
}

/// Receives and decrypts a CrypRQ record from the transport
///
/// As specified in Section 6.1-6.2:
/// - Decodes record header + ciphertext
/// - Looks up keys by epoch and direction
/// - Reconstructs nonce and decrypts
///
/// # Arguments
///
/// * `buf` - Raw bytes from transport
/// * `keys` - Directional keys for decryption
///
/// # Returns
///
/// * `(record_header, plaintext)` - Decrypted record and plaintext
pub fn recv_record(buf: &[u8], keys: &DirectionKeys) -> io::Result<(RecordHeader, Vec<u8>)> {
    // Decode record
    let record = Record::from_bytes(buf)?;

    // Decrypt using directional keys
    let plaintext = record.decrypt(&keys.key, &keys.iv)?;

    Ok((record.header, plaintext))
}

/// Stream ID for VPN/TUN traffic
pub const VPN_STREAM_ID: u32 = 1;

/// Stream ID allocation counter (simple implementation)
/// In production, this would be more sophisticated (per-peer, etc.)
static STREAM_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(2);

/// Allocates a new stream ID for file transfer or other purposes
pub fn alloc_stream_id() -> u32 {
    STREAM_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_recv_record_roundtrip() {
        let epoch = Epoch::initial();
        let stream_id = 1;
        let seq = 1;
        let plaintext = b"Hello, CrypRQ!";
        let keys = DirectionKeys {
            key: [0x42; 32],
            iv: [0x24; 12],
        };

        // Send
        let encoded = send_record(epoch, stream_id, seq, MSG_TYPE_DATA, 0, plaintext, &keys)
            .expect("send_record should not fail in test");

        // Receive
        let (header, decrypted) =
            recv_record(&encoded, &keys).expect("recv_record should not fail in test");

        assert_eq!(header.message_type, MSG_TYPE_DATA);
        assert_eq!(header.stream_id, stream_id);
        assert_eq!(header.sequence_number, seq);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_stream_id_allocation() {
        let id1 = alloc_stream_id();
        let id2 = alloc_stream_id();
        assert!(id2 > id1);
    }
}
