// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! CrypRQ v1.0 Record Layer
//!
//! Implements the record structure as specified in cryp-rq-protocol-v1.md Section 6.1

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use std::io::{self, Read, Write};

/// Protocol version for CrypRQ v1.0
pub const PROTOCOL_VERSION: u8 = 0x01;

/// Record header size (20 bytes) as specified in Section 6.1.1
pub const RECORD_HEADER_SIZE: usize = 20;

/// Message type: Generic stream/tunnel data
pub const MSG_TYPE_DATA: u8 = 0x01;

/// Message type: File metadata
pub const MSG_TYPE_FILE_META: u8 = 0x02;

/// Message type: File chunk
pub const MSG_TYPE_FILE_CHUNK: u8 = 0x03;

/// Message type: File acknowledgment
pub const MSG_TYPE_FILE_ACK: u8 = 0x04;

/// Message type: VPN packet
pub const MSG_TYPE_VPN_PACKET: u8 = 0x05;

/// Message type: Control message
pub const MSG_TYPE_CONTROL: u8 = 0x10;

/// CrypRQ record header structure (20 bytes)
///
/// As specified in Section 6.1.1:
/// - Version (1 byte)
/// - Message Type (1 byte)
/// - Flags (1 byte)
/// - Epoch (1 byte, 8-bit)
/// - Stream ID (4 bytes, big-endian)
/// - Sequence Number (8 bytes, big-endian)
/// - Ciphertext Length (4 bytes, big-endian)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordHeader {
    pub version: u8,
    pub message_type: u8,
    pub flags: u8,
    pub epoch: u8,
    pub stream_id: u32,
    pub sequence_number: u64,
    pub ciphertext_length: u32,
}

impl RecordHeader {
    /// Creates a new record header
    pub fn new(
        message_type: u8,
        flags: u8,
        epoch: u8,
        stream_id: u32,
        sequence_number: u64,
        ciphertext_length: u32,
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            message_type,
            flags,
            epoch,
            stream_id,
            sequence_number,
            ciphertext_length,
        }
    }

    /// Serializes the header to bytes (big-endian)
    pub fn to_bytes(&self) -> [u8; RECORD_HEADER_SIZE] {
        let mut buf = [0u8; RECORD_HEADER_SIZE];
        buf[0] = self.version;
        buf[1] = self.message_type;
        buf[2] = self.flags;
        buf[3] = self.epoch;
        buf[4..8].copy_from_slice(&self.stream_id.to_be_bytes());
        buf[8..16].copy_from_slice(&self.sequence_number.to_be_bytes());
        buf[16..20].copy_from_slice(&self.ciphertext_length.to_be_bytes());
        buf
    }

    /// Deserializes the header from bytes (big-endian)
    pub fn from_bytes(buf: &[u8]) -> io::Result<Self> {
        if buf.len() < RECORD_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Buffer too short for record header",
            ));
        }

        let version = buf[0];
        if version != PROTOCOL_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported protocol version: {}", version),
            ));
        }

        let message_type = buf[1];
        let flags = buf[2];
        let epoch = buf[3];
        let stream_id = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let sequence_number = u64::from_be_bytes([
            buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
        ]);
        let ciphertext_length = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);

        Ok(Self {
            version,
            message_type,
            flags,
            epoch,
            stream_id,
            sequence_number,
            ciphertext_length,
        })
    }

    /// Reads the header from a reader
    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; RECORD_HEADER_SIZE];
        reader.read_exact(&mut buf)?;
        Self::from_bytes(&buf)
    }

    /// Writes the header to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_bytes())?;
        Ok(())
    }
}

/// Complete CrypRQ record (header + ciphertext)
#[derive(Debug, Clone)]
pub struct Record {
    pub header: RecordHeader,
    pub ciphertext: Vec<u8>,
}

impl Record {
    /// Creates a new record
    pub fn new(
        message_type: u8,
        flags: u8,
        epoch: u8,
        stream_id: u32,
        sequence_number: u64,
        ciphertext: Vec<u8>,
    ) -> Self {
        let header = RecordHeader::new(
            message_type,
            flags,
            epoch,
            stream_id,
            sequence_number,
            ciphertext.len() as u32,
        );
        Self { header, ciphertext }
    }

    /// Serializes the entire record (header + ciphertext)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(RECORD_HEADER_SIZE + self.ciphertext.len());
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.ciphertext);
        buf
    }

    /// Deserializes a record from bytes
    pub fn from_bytes(buf: &[u8]) -> io::Result<Self> {
        let header = RecordHeader::from_bytes(&buf[..RECORD_HEADER_SIZE])?;
        let ciphertext = buf[RECORD_HEADER_SIZE..].to_vec();

        if ciphertext.len() != header.ciphertext_length as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Ciphertext length mismatch: expected {}, got {}",
                    header.ciphertext_length,
                    ciphertext.len()
                ),
            ));
        }

        Ok(Self { header, ciphertext })
    }

    /// Reads a record from a reader
    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let header = RecordHeader::read_from(reader)?;
        let mut ciphertext = vec![0u8; header.ciphertext_length as usize];
        reader.read_exact(&mut ciphertext)?;
        Ok(Self { header, ciphertext })
    }

    /// Writes a record to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.header.write_to(writer)?;
        writer.write_all(&self.ciphertext)?;
        Ok(())
    }

    /// Encrypts plaintext and creates a record
    ///
    /// As specified in Section 6.2:
    /// - Constructs nonce using TLS 1.3-style XOR
    /// - Encrypts plaintext with AEAD
    /// - Uses header as AAD
    #[allow(clippy::too_many_arguments)]
    pub fn encrypt(
        _version: u8,
        message_type: u8,
        flags: u8,
        epoch: u8,
        stream_id: u32,
        sequence_number: u64,
        plaintext: &[u8],
        key: &[u8; 32],
        static_iv: &[u8; 12],
    ) -> io::Result<Self> {
        // Construct nonce using TLS 1.3-style XOR
        let seq_be = sequence_number.to_be_bytes();
        let mut nonce_bytes = *static_iv;
        for i in 0..8 {
            nonce_bytes[4 + i] ^= seq_be[i];
        }
        let nonce = Nonce::from(nonce_bytes);

        // Create cipher
        let cipher = ChaCha20Poly1305::new(key.into());

        // Calculate ciphertext length (plaintext + 16-byte Poly1305 tag)
        const POLY1305_TAG_SIZE: usize = 16;
        let ciphertext_length = (plaintext.len() + POLY1305_TAG_SIZE) as u32;

        // Create header with ciphertext length (as per spec Section 6.1)
        let header = RecordHeader::new(
            message_type,
            flags,
            epoch,
            stream_id,
            sequence_number,
            ciphertext_length,
        );

        // Encrypt with header as AAD (as per spec Section 6.2)
        let header_bytes = header.to_bytes();
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: &header_bytes,
                },
            )
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Encryption failed: {}", e),
                )
            })?;

        Ok(Self { header, ciphertext })
    }

    /// Decrypts the record and returns plaintext
    ///
    /// As specified in Section 6.2:
    /// - Reconstructs nonce using TLS 1.3-style XOR
    /// - Decrypts with AEAD using header as AAD
    pub fn decrypt(&self, key: &[u8; 32], static_iv: &[u8; 12]) -> io::Result<Vec<u8>> {
        // Reconstruct nonce using TLS 1.3-style XOR
        let seq_be = self.header.sequence_number.to_be_bytes();
        let mut nonce_bytes = *static_iv;
        for i in 0..8 {
            nonce_bytes[4 + i] ^= seq_be[i];
        }
        let nonce = Nonce::from(nonce_bytes);

        // Create cipher
        let cipher = ChaCha20Poly1305::new(key.into());

        // Decrypt with header as AAD
        let header_bytes = self.header.to_bytes();
        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &self.ciphertext,
                    aad: &header_bytes,
                },
            )
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Decryption failed: {}", e),
                )
            })?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_serialization() {
        let header = RecordHeader::new(MSG_TYPE_DATA, 0x00, 0, 1, 42, 100);

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), RECORD_HEADER_SIZE);
        assert_eq!(bytes[0], PROTOCOL_VERSION);
        assert_eq!(bytes[1], MSG_TYPE_DATA);
        assert_eq!(bytes[3], 0); // epoch

        let deserialized = RecordHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_record_serialization() {
        let ciphertext = vec![0x01, 0x02, 0x03, 0x04];
        let record = Record::new(MSG_TYPE_DATA, 0x00, 0, 1, 42, ciphertext.clone());

        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), RECORD_HEADER_SIZE + ciphertext.len());

        let deserialized = Record::from_bytes(&bytes).unwrap();
        assert_eq!(record.header, deserialized.header);
        assert_eq!(record.ciphertext, deserialized.ciphertext);
    }

    #[test]
    fn test_epoch_wrapping() {
        // Test that epoch is 8-bit
        let header = RecordHeader::new(MSG_TYPE_DATA, 0, 255, 1, 0, 0);
        let bytes = header.to_bytes();
        assert_eq!(bytes[3], 255);

        let header2 = RecordHeader::new(MSG_TYPE_DATA, 0, 0, 1, 0, 0);
        let bytes2 = header2.to_bytes();
        assert_eq!(bytes2[3], 0);
    }
}
