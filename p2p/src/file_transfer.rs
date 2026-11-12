// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use anyhow::{Context, Result};
use futures::{AsyncRead, AsyncWrite};
use libp2p::{request_response::Codec, StreamProtocol};
use sha2::{Digest, Sha256};
use std::io;
use std::path::PathBuf;

/// File transfer protocol identifier
pub const FILE_PROTOCOL: StreamProtocol = StreamProtocol::new("/cryprq/file/1.0.0");

/// Packet types
const PACKET_TYPE_METADATA: u32 = 0;
const PACKET_TYPE_DATA: u32 = 1;
const PACKET_TYPE_END: u32 = 2;

/// Chunk size for file transfer (64KB)
pub const CHUNK_SIZE: usize = 65536;

/// File metadata
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

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&PACKET_TYPE_METADATA.to_be_bytes());
        let filename_bytes = self.filename.as_bytes();
        buf.extend_from_slice(&(filename_bytes.len() as u32).to_be_bytes());
        buf.extend_from_slice(filename_bytes);
        buf.extend_from_slice(&self.size.to_be_bytes());
        buf.extend_from_slice(&self.hash);
        buf
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
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

/// Data chunk packet
pub struct DataChunk {
    pub chunk_id: u32,
    pub data: Vec<u8>,
}

impl DataChunk {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&PACKET_TYPE_DATA.to_be_bytes());
        buf.extend_from_slice(&self.chunk_id.to_be_bytes());
        buf.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            anyhow::bail!("Data chunk packet too short");
        }
        let packet_type = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if packet_type != PACKET_TYPE_DATA {
            anyhow::bail!("Invalid packet type for data chunk");
        }
        let chunk_id = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let data_len = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;
        if data.len() < 12 + data_len {
            anyhow::bail!("Data chunk packet too short for data");
        }
        let chunk_data = data[12..12 + data_len].to_vec();
        Ok(Self {
            chunk_id,
            data: chunk_data,
        })
    }
}

/// End packet
pub fn create_end_packet() -> Vec<u8> {
    PACKET_TYPE_END.to_be_bytes().to_vec()
}

/// Check if packet is end packet
pub fn is_end_packet(data: &[u8]) -> bool {
    data.len() >= 4 && u32::from_be_bytes([data[0], data[1], data[2], data[3]]) == PACKET_TYPE_END
}

/// Calculate SHA-256 hash of file
pub fn calculate_file_hash(file_path: &PathBuf) -> Result<[u8; 32]> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(file_path).context("Failed to open file")?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK_SIZE];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.finalize().into())
}

/// File transfer codec
#[derive(Clone, Default)]
pub struct FileTransferCodec;

#[async_trait::async_trait]
impl Codec for FileTransferCodec {
    type Protocol = String;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        if len > 10 * 1024 * 1024 {
            // Max 10MB per packet
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Packet too large",
            ));
        }
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        if len > 1024 {
            // Response is just acknowledgment, should be small
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Response too large",
            ));
        }
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let len = req.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&req).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let len = res.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&res).await?;
        io.flush().await?;
        Ok(())
    }
}
