// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! File transfer manager for CrypRQ record layer
//!
//! Manages file transfers using stream IDs and routes FILE_* records to appropriate handlers

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use tokio::sync::mpsc;

/// File metadata (duplicated from p2p to avoid dependency cycle)
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub filename: String,
    pub size: u64,
    pub hash: [u8; 32],
}

impl FileMetadata {
    pub fn new(filename: String, size: u64, hash: [u8; 32]) -> Self {
        Self {
            filename,
            size,
            hash,
        }
    }

    /// Serialize file metadata (compatible with p2p::file_transfer::FileMetadata format)
    pub fn serialize(&self) -> Vec<u8> {
        const PACKET_TYPE_METADATA: u32 = 0;
        let mut buf = Vec::new();
        buf.extend_from_slice(&PACKET_TYPE_METADATA.to_be_bytes());
        let filename_bytes = self.filename.as_bytes();
        buf.extend_from_slice(&(filename_bytes.len() as u32).to_be_bytes());
        buf.extend_from_slice(filename_bytes);
        buf.extend_from_slice(&self.size.to_be_bytes());
        buf.extend_from_slice(&self.hash);
        buf
    }

    /// Deserialize file metadata (compatible with p2p::file_transfer::FileMetadata format)
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        const PACKET_TYPE_METADATA: u32 = 0;
        if data.len() < 4 {
            anyhow::bail!("Metadata packet too short");
        }
        let packet_type = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if packet_type != PACKET_TYPE_METADATA {
            anyhow::bail!("Invalid packet type for metadata");
        }
        let mut offset = 4;

        // Read filename length
        if data.len() < offset + 4 {
            anyhow::bail!("Metadata packet too short for filename length");
        }
        let filename_len = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        // Read filename
        if data.len() < offset + filename_len {
            anyhow::bail!("Metadata packet too short for filename");
        }
        let filename = String::from_utf8(data[offset..offset + filename_len].to_vec())
            .context("Invalid UTF-8 in filename")?;
        offset += filename_len;

        // Read file size
        if data.len() < offset + 8 {
            anyhow::bail!("Metadata packet too short for file size");
        }
        let size = u64::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Read hash
        if data.len() < offset + 32 {
            anyhow::bail!("Metadata packet too short for hash");
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&data[offset..offset + 32]);

        Ok(Self {
            filename,
            size,
            hash,
        })
    }
}

/// Incoming file transfer state
#[derive(Debug)]
struct IncomingTransfer {
    metadata: FileMetadata,
    output_path: PathBuf,
    file: Option<File>,
    bytes_received: u64,
    chunks_received: Vec<u32>,
}

/// Outgoing file transfer state
#[derive(Debug)]
struct OutgoingTransfer {
    metadata: FileMetadata,
    file_path: PathBuf,
    chunks_sent: u32,
    total_chunks: u32,
}

/// File transfer manager
pub struct FileTransferManager {
    next_stream_id: AtomicU32,
    incoming: Mutex<HashMap<u32, IncomingTransfer>>,
    outgoing: Mutex<HashMap<u32, OutgoingTransfer>>,
    output_dir: PathBuf,
}

impl FileTransferManager {
    /// Create a new file transfer manager
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            next_stream_id: AtomicU32::new(2), // Start at 2 (1 is VPN_STREAM_ID)
            incoming: Mutex::new(HashMap::new()),
            outgoing: Mutex::new(HashMap::new()),
            output_dir,
        }
    }

    /// Allocate a new stream ID for a file transfer
    pub fn alloc_stream_id(&self) -> u32 {
        self.next_stream_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Start an incoming file transfer
    pub fn start_incoming_transfer(
        &self,
        stream_id: u32,
        metadata: FileMetadata,
    ) -> Result<()> {
        let output_path = self.output_dir.join(&metadata.filename);

        // Create file for writing
        let file = File::create(&output_path)
            .with_context(|| format!("Failed to create output file: {:?}", output_path))?;

        let transfer = IncomingTransfer {
            metadata,
            output_path,
            file: Some(file),
            bytes_received: 0,
            chunks_received: Vec::new(),
        };

        let mut incoming = self.incoming.lock().unwrap();
        incoming.insert(stream_id, transfer);

        log::info!(
            "Started incoming file transfer: stream_id={}, filename={}, size={}",
            stream_id,
            incoming.get(&stream_id).unwrap().metadata.filename,
            incoming.get(&stream_id).unwrap().metadata.size
        );

        Ok(())
    }

    /// Write a chunk to an incoming transfer
    pub fn write_chunk(&self, stream_id: u32, chunk_data: &[u8]) -> Result<()> {
        let mut incoming = self.incoming.lock().unwrap();
        let transfer = incoming
            .get_mut(&stream_id)
            .ok_or_else(|| anyhow::anyhow!("No incoming transfer for stream_id {}", stream_id))?;

        if let Some(ref mut file) = transfer.file {
            file.write_all(chunk_data)
                .context("Failed to write chunk to file")?;
            transfer.bytes_received += chunk_data.len() as u64;

            log::debug!(
                "Wrote chunk: stream_id={}, size={}, total={}/{}",
                stream_id,
                chunk_data.len(),
                transfer.bytes_received,
                transfer.metadata.size
            );

            // Check if transfer is complete
            if transfer.bytes_received >= transfer.metadata.size {
                // Close file
                transfer.file.take();
                log::info!(
                    "File transfer complete: stream_id={}, filename={}",
                    stream_id,
                    transfer.metadata.filename
                );
                // TODO: Verify SHA-256 hash
            }
        }

        Ok(())
    }

    /// Handle file acknowledgment
    pub fn handle_ack(&self, stream_id: u32, _ack_data: &[u8]) -> Result<()> {
        log::debug!("Received file ACK for stream_id={}", stream_id);
        // TODO: Implement ACK handling (retransmissions, congestion control, etc.)
        Ok(())
    }

    /// Handle control message
    pub fn handle_control(&self, stream_id: u32, control_data: &[u8]) -> Result<()> {
        log::debug!(
            "Received control message: stream_id={}, size={}",
            stream_id,
            control_data.len()
        );
        // TODO: Handle control messages (end-of-file, error, etc.)
        Ok(())
    }

    /// Register an outgoing transfer
    pub fn register_outgoing(&self, stream_id: u32, metadata: FileMetadata, file_path: PathBuf) {
        const CHUNK_SIZE: usize = 65536; // 64KB
        let total_chunks = ((metadata.size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE) as u32;
        let transfer = OutgoingTransfer {
            metadata,
            file_path,
            chunks_sent: 0,
            total_chunks,
        };
        let mut outgoing = self.outgoing.lock().unwrap();
        outgoing.insert(stream_id, transfer);
    }
}

