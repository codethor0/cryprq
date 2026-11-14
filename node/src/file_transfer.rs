// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use anyhow::{Context, Result};
use ring::rand::SecureRandom;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Largest file a peer may announce; larger transfers are rejected before any disk write.
pub const MAX_INCOMING_FILE_SIZE: u64 = 1 << 30; // 1 GiB
/// Bound open files and receiver state even when a peer abandons transfers.
pub const MAX_ACTIVE_INCOMING_TRANSFERS: usize = 64;
/// Bound the total announced size of active transfers, including unwritten bytes.
pub const MAX_RESERVED_INCOMING_BYTES: u64 = 2 << 30; // 2 GiB
/// Reclaim stalled transfers even when no further receiver operation occurs.
pub const MAX_INCOMING_IDLE: Duration = Duration::from_secs(120);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
struct CleanupPolicy {
    idle: Duration,
    interval: Duration,
}

type CleanupSpawner = fn(Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>>;

fn spawn_cleanup(task: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("cryprq-file-cleanup".into())
        .spawn(task)
}

struct CleanupWorker {
    stop: Arc<(Mutex<bool>, Condvar)>,
    handle: JoinHandle<()>,
}

impl CleanupWorker {
    fn stop(self) {
        {
            let mut stopped = self
                .stop
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            *stopped = true;
            self.stop.1.notify_one();
        }
        // No owner mutex is held while waiting for the worker to finish.
        if self.handle.join().is_err() {
            log::error!("File cleanup worker panicked during shutdown");
        }
    }
}

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
struct IncomingTransfer {
    metadata: FileMetadata,
    output_path: PathBuf,
    temporary_path: PathBuf,
    file: Option<File>,
    hasher: ring::digest::Context,
    bytes_received: u64,
    last_progress: Instant,
    cleanup_pending: bool,
    cleanup_error: Option<std::io::ErrorKind>,
    redacted_logs: bool,
    #[allow(dead_code)]
    chunks_received: Vec<u32>,
}

impl Drop for IncomingTransfer {
    fn drop(&mut self) {
        if !self.try_cleanup() {
            if self.redacted_logs {
                log::error!(
                    "Receiver dropped with undeleted temporary file; operator cleanup required: error_kind={:?}",
                    self.cleanup_error
                );
            } else {
                log::error!(
                    "Receiver dropped with undeleted temporary file; operator cleanup required: {:?}",
                    self.temporary_path
                );
            }
        }
    }
}

impl IncomingTransfer {
    fn try_cleanup(&mut self) -> bool {
        // Close before unlinking for platforms that cannot remove open files.
        self.file.take();
        match std::fs::remove_file(&self.temporary_path) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(error) => {
                // Persistent failures remain charged without producing a log storm.
                if self.cleanup_error != Some(error.kind()) {
                    if self.redacted_logs {
                        log::warn!("Temporary cleanup pending: error_kind={:?}", error.kind());
                    } else {
                        log::warn!(
                            "Temporary cleanup pending for {:?}: {}",
                            self.temporary_path,
                            error
                        );
                    }
                    self.cleanup_error = Some(error.kind());
                }
                false
            }
        }
    }
}

/// Outgoing file transfer state
#[derive(Debug)]
struct OutgoingTransfer {
    #[allow(dead_code)]
    metadata: FileMetadata,
    #[allow(dead_code)]
    file_path: PathBuf,
    #[allow(dead_code)]
    chunks_sent: u32,
    #[allow(dead_code)]
    total_chunks: u32,
}

/// File transfer manager.
///
/// At most one cleanup worker is started on first admission. It checks inactivity
/// every second independently of traffic; admission and chunk operations also
/// perform cleanup. Files that cannot be unlinked remain quarantined and charged
/// against both limits until a retry succeeds. Dropping the manager stops and
/// joins the worker, then makes a final cleanup attempt and logs OS failures.
/// The reservation limit covers temporary files, including aliases awaiting
/// unlink after publication. Verified files are otherwise outside this quota.
pub struct FileTransferManager {
    next_stream_id: AtomicU32,
    incoming: Arc<Mutex<HashMap<u32, IncomingTransfer>>>,
    outgoing: Mutex<HashMap<u32, OutgoingTransfer>>,
    output_dir: PathBuf,
    cleanup_policy: CleanupPolicy,
    cleanup_spawner: CleanupSpawner,
    cleanup_worker: Mutex<Option<CleanupWorker>>,
    redacted_logs: bool,
}

impl Drop for FileTransferManager {
    fn drop(&mut self) {
        let worker = self
            .cleanup_worker
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(worker) = worker {
            worker.stop();
        }
    }
}

impl FileTransferManager {
    /// Create a new file transfer manager
    pub fn new(output_dir: PathBuf) -> Self {
        Self::with_cleanup_options(
            output_dir,
            CleanupPolicy {
                idle: MAX_INCOMING_IDLE,
                interval: CLEANUP_INTERVAL,
            },
            spawn_cleanup,
        )
    }

    /// Create a receiver whose logs omit peer-supplied filenames, filesystem
    /// paths and OS error details. File safety and admission limits are unchanged.
    pub fn new_redacted(output_dir: PathBuf) -> Self {
        let mut manager = Self::new(output_dir);
        manager.redacted_logs = true;
        manager
    }

    fn with_cleanup_options(
        output_dir: PathBuf,
        cleanup_policy: CleanupPolicy,
        cleanup_spawner: CleanupSpawner,
    ) -> Self {
        Self {
            next_stream_id: AtomicU32::new(2), // Start at 2 (1 is VPN_STREAM_ID)
            incoming: Arc::new(Mutex::new(HashMap::new())),
            outgoing: Mutex::new(HashMap::new()),
            output_dir,
            cleanup_policy,
            cleanup_spawner,
            cleanup_worker: Mutex::new(None),
            redacted_logs: false,
        }
    }

    fn ensure_cleanup_worker(&self) -> Result<()> {
        let mut worker = self
            .cleanup_worker
            .lock()
            .map_err(|error| anyhow::anyhow!("Cleanup worker lock poisoned: {error}"))?;
        if worker.is_some() {
            return Ok(());
        }
        let incoming = Arc::downgrade(&self.incoming);
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_stop = Arc::clone(&stop);
        let policy = self.cleanup_policy;
        let redacted_logs = self.redacted_logs;
        let handle = (self.cleanup_spawner)(Box::new(move || loop {
            let stopped = worker_stop
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let (stopped, _) = worker_stop
                .1
                .wait_timeout_while(stopped, policy.interval, |stopped| !*stopped)
                .unwrap_or_else(|error| error.into_inner());
            if *stopped {
                break;
            }
            drop(stopped);
            let Some(incoming) = incoming.upgrade() else {
                break;
            };
            match incoming.lock() {
                Ok(mut transfers) => {
                    Self::expire_inactive(&mut transfers, Instant::now(), policy.idle)
                }
                Err(error) => {
                    if redacted_logs {
                        log::error!("File cleanup state lock poisoned");
                    } else {
                        log::error!("File cleanup state lock poisoned: {error}");
                    }
                    break;
                }
            };
        }))
        .context("Failed to start file cleanup worker; transfer not admitted")?;
        *worker = Some(CleanupWorker { stop, handle });
        Ok(())
    }

    /// Allocate a new stream ID for a file transfer
    pub fn alloc_stream_id(&self) -> u32 {
        self.next_stream_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Start an incoming file transfer
    pub fn start_incoming_transfer(&self, stream_id: u32, metadata: FileMetadata) -> Result<()> {
        self.start_incoming_transfer_with_clock(stream_id, metadata, Instant::now)
    }

    fn start_incoming_transfer_with_clock(
        &self,
        stream_id: u32,
        metadata: FileMetadata,
        clock: impl Fn() -> Instant,
    ) -> Result<()> {
        let mut incoming = self
            .incoming
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;
        Self::expire_inactive(&mut incoming, clock(), self.cleanup_policy.idle);
        // The filename is peer-controlled; only accept a single plain path component.
        let mut components = Path::new(&metadata.filename).components();
        if !matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(_)), None)
        ) {
            anyhow::bail!("Rejected unsafe filename: {:?}", metadata.filename);
        }
        if metadata.size > MAX_INCOMING_FILE_SIZE {
            anyhow::bail!(
                "Rejected transfer of {} bytes (limit {})",
                metadata.size,
                MAX_INCOMING_FILE_SIZE
            );
        }
        let output_path = self.output_dir.join(&metadata.filename);
        let filename = metadata.filename.clone();
        let size = metadata.size;

        if incoming.contains_key(&stream_id) {
            anyhow::bail!("Transfer already active for stream_id {stream_id}");
        }
        if incoming.len() >= MAX_ACTIVE_INCOMING_TRANSFERS {
            anyhow::bail!("Too many active incoming transfers");
        }
        let reserved = incoming
            .values()
            .try_fold(0u64, |total, transfer| {
                total.checked_add(transfer.metadata.size)
            })
            .and_then(|total| total.checked_add(size));
        if !matches!(reserved, Some(total) if total <= MAX_RESERVED_INCOMING_BYTES) {
            anyhow::bail!("Incoming transfer exceeds aggregate reserved storage limit");
        }
        // Reject existing files and dangling symlinks now; final publication also
        // checks atomically, because this path may change while receiving bytes.
        match std::fs::symlink_metadata(&output_path) {
            Ok(_) => anyhow::bail!("Output file already exists: {output_path:?}"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("Failed to inspect output path"),
        }
        self.ensure_cleanup_worker()?;

        let mut random = [0u8; 16];
        ring::rand::SystemRandom::new()
            .fill(&mut random)
            .map_err(|_| anyhow::anyhow!("Failed to generate temporary filename"))?;
        let temporary_name = hex::encode(random);
        let temporary_path = self
            .output_dir
            .join(format!(".cryprq-{temporary_name}.part"));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(&temporary_path)
            .context("Failed to create private temporary file")?;

        let transfer = IncomingTransfer {
            metadata,
            output_path,
            temporary_path,
            file: Some(file),
            hasher: ring::digest::Context::new(&ring::digest::SHA256),
            bytes_received: 0,
            last_progress: clock(),
            cleanup_pending: false,
            cleanup_error: None,
            redacted_logs: self.redacted_logs,
            chunks_received: Vec::new(),
        };

        incoming.insert(stream_id, transfer);
        // No chunk will ever arrive for an empty file, so verify it now.
        if size == 0 {
            return Self::finish(stream_id, &mut incoming);
        }

        if self.redacted_logs {
            log::info!("Started incoming file transfer: stream_id={stream_id}, size={size}");
        } else {
            log::info!(
                "Started incoming file transfer: stream_id={stream_id}, filename={filename}, size={size}"
            );
        }

        Ok(())
    }

    /// Write a chunk to an incoming transfer
    pub fn write_chunk(&self, stream_id: u32, chunk_data: &[u8]) -> Result<()> {
        self.write_chunk_with_clock(stream_id, chunk_data, Instant::now)
    }

    fn write_chunk_with_clock(
        &self,
        stream_id: u32,
        chunk_data: &[u8],
        clock: impl Fn() -> Instant,
    ) -> Result<()> {
        let mut incoming = self
            .incoming
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;
        Self::expire_inactive(&mut incoming, clock(), self.cleanup_policy.idle);
        let transfer = incoming
            .get_mut(&stream_id)
            .ok_or_else(|| anyhow::anyhow!("No incoming transfer for stream_id {stream_id}"))?;
        if transfer.cleanup_pending {
            anyhow::bail!("Transfer awaiting temporary cleanup for stream_id {stream_id}");
        }

        if let Some(ref mut file) = transfer.file {
            let new_total = transfer
                .bytes_received
                .checked_add(chunk_data.len() as u64)
                .filter(|total| *total <= transfer.metadata.size);
            let Some(new_total) = new_total else {
                return Err(Self::discard(
                    &mut incoming,
                    stream_id,
                    "received more data than announced",
                ));
            };
            if let Err(error) = file.write_all(chunk_data) {
                return Err(Self::discard(
                    &mut incoming,
                    stream_id,
                    &format!("Failed to write chunk: {error}"),
                ));
            }
            transfer.hasher.update(chunk_data);
            transfer.bytes_received = new_total;
            if !chunk_data.is_empty() {
                transfer.last_progress = clock();
            }

            log::debug!(
                "Wrote chunk: stream_id={}, size={}, total={}/{}",
                stream_id,
                chunk_data.len(),
                transfer.bytes_received,
                transfer.metadata.size
            );

            // Check if transfer is complete
            if transfer.bytes_received == transfer.metadata.size {
                return Self::finish(stream_id, &mut incoming);
            }
        }

        Ok(())
    }

    fn expire_inactive(
        incoming: &mut HashMap<u32, IncomingTransfer>,
        now: Instant,
        idle: Duration,
    ) {
        incoming.retain(|stream_id, transfer| {
            if !transfer.cleanup_pending
                && now.saturating_duration_since(transfer.last_progress) >= idle
            {
                log::warn!(
                    "Incoming transfer expired without byte progress: stream_id={stream_id}"
                );
                transfer.cleanup_pending = true;
            }
            // Failed unlink keeps the entry and its reservation, with no open file.
            !transfer.cleanup_pending || !transfer.try_cleanup()
        });
    }

    /// Close a completed transfer and verify its SHA-256 against the announced hash
    fn finish(stream_id: u32, incoming: &mut HashMap<u32, IncomingTransfer>) -> Result<()> {
        let transfer = incoming
            .get_mut(&stream_id)
            .expect("transfer present while locked");
        let result = (|| -> Result<()> {
            if let Some(file) = transfer.file.take() {
                let flushed = file.sync_all();
                drop(file);
                flushed.context("Failed to flush received file")?;
            }
            let digest = transfer.hasher.clone().finish();
            if digest.as_ref() != transfer.metadata.hash.as_slice() {
                anyhow::bail!("SHA-256 mismatch");
            }
            // A same-directory hard link publishes the verified inode atomically and
            // fails if the destination appeared meanwhile; rename could overwrite it.
            std::fs::hard_link(&transfer.temporary_path, &transfer.output_path)
                .context("Failed to publish verified file without overwriting destination")?;
            if transfer.redacted_logs {
                log::info!("File transfer complete: stream_id={stream_id}, sha256=verified");
            } else {
                log::info!(
                    "File transfer complete: stream_id={}, filename={}, sha256=verified",
                    stream_id,
                    transfer.metadata.filename
                );
            }
            Ok(())
        })();
        transfer.cleanup_pending = true;
        if transfer.try_cleanup() {
            incoming.remove(&stream_id);
        }
        result
    }

    /// Remove a partially written or corrupted file so it cannot be mistaken for a valid one
    fn discard(
        incoming: &mut HashMap<u32, IncomingTransfer>,
        stream_id: u32,
        reason: &str,
    ) -> anyhow::Error {
        let transfer = incoming
            .get_mut(&stream_id)
            .expect("transfer present while locked");
        let filename = transfer.metadata.filename.clone();
        transfer.cleanup_pending = true;
        let removed = transfer.try_cleanup();
        if transfer.redacted_logs {
            log::warn!("File transfer rejected: stream_id={stream_id}, removed={removed}");
        } else {
            log::warn!(
                "File transfer rejected: filename={filename}, reason={reason}, removed={removed}"
            );
        }
        if removed {
            incoming.remove(&stream_id);
        }
        anyhow::anyhow!("File transfer rejected for {filename}: {reason}")
    }

    /// Handle file acknowledgment
    pub fn handle_ack(&self, stream_id: u32, _ack_data: &[u8]) -> Result<()> {
        log::debug!("Received file ACK for stream_id={stream_id}");
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
        let total_chunks = (metadata.size as usize).div_ceil(CHUNK_SIZE) as u32;
        let transfer = OutgoingTransfer {
            metadata,
            file_path,
            chunks_sent: 0,
            total_chunks,
        };
        let mut outgoing = self
            .outgoing
            .lock()
            .expect("Outgoing transfers lock poisoned");
        outgoing.insert(stream_id, transfer);
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // Tests can use unwrap/expect
mod tests {
    use super::*;

    fn sha256(data: &[u8]) -> [u8; 32] {
        ring::digest::digest(&ring::digest::SHA256, data)
            .as_ref()
            .try_into()
            .unwrap()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cryprq-ft-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn wait_until(condition: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !condition() {
            assert!(
                Instant::now() < deadline,
                "cleanup did not make bounded progress"
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn timed_manager(dir: PathBuf, idle: Duration) -> FileTransferManager {
        FileTransferManager::with_cleanup_options(
            dir,
            CleanupPolicy {
                idle,
                interval: Duration::from_millis(5),
            },
            spawn_cleanup,
        )
    }

    #[test]
    fn worker_reclaims_idle_transfer_without_another_receiver_operation() {
        let dir = temp_dir("worker-idle");
        let manager = timed_manager(dir.clone(), Duration::from_millis(25));
        manager
            .start_incoming_transfer(2, FileMetadata::new("idle.bin".into(), 2, sha256(b"ab")))
            .unwrap();
        manager.write_chunk(2, b"a").unwrap();
        let temporary_path = manager
            .incoming
            .lock()
            .unwrap()
            .get(&2)
            .unwrap()
            .temporary_path
            .clone();
        // Only filesystem observations follow: the worker, not a receiver call,
        // must close and remove the abandoned transfer.
        wait_until(|| !temporary_path.exists());
        assert!(manager.incoming.lock().unwrap().is_empty());
        assert!(!dir.join("idle.bin").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn worker_treats_transfer_progress_independently() {
        let dir = temp_dir("worker-independent");
        let manager = timed_manager(dir.clone(), MAX_INCOMING_IDLE);
        for (stream_id, filename) in [(2, "stalled.bin"), (3, "progress.bin")] {
            manager
                .start_incoming_transfer(
                    stream_id,
                    FileMetadata::new(filename.into(), 2, sha256(b"ab")),
                )
                .unwrap();
        }
        manager.write_chunk(3, b"a").unwrap();
        {
            let mut incoming = manager.incoming.lock().unwrap();
            // Inject only the stalled transfer's elapsed inactivity, avoiding a
            // 120-second test sleep while retaining the production worker path.
            incoming.get_mut(&2).unwrap().last_progress = Instant::now() - MAX_INCOMING_IDLE;
        }
        wait_until(|| !manager.incoming.lock().unwrap().contains_key(&2));
        let incoming = manager.incoming.lock().unwrap();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming.get(&3).unwrap().bytes_received, 1);
        drop(incoming);
        manager.write_chunk(3, b"b").unwrap();
        assert_eq!(std::fs::read(dir.join("progress.bin")).unwrap(), b"ab");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn worker_start_failure_rejects_admission_before_creating_a_file() {
        fn fail_spawn(_task: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
            Err(std::io::Error::other("injected worker start failure"))
        }
        let dir = temp_dir("worker-start-error");
        let manager = FileTransferManager::with_cleanup_options(
            dir.clone(),
            CleanupPolicy {
                idle: MAX_INCOMING_IDLE,
                interval: CLEANUP_INTERVAL,
            },
            fail_spawn,
        );
        let error = manager
            .start_incoming_transfer(2, FileMetadata::new("blocked.bin".into(), 1, sha256(b"x")))
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("Failed to start file cleanup worker"));
        assert!(manager.incoming.lock().unwrap().is_empty());
        assert!(manager.cleanup_worker.lock().unwrap().is_none());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn drop_wakes_and_joins_worker_without_waiting_for_its_poll_interval() {
        let dir = temp_dir("worker-shutdown");
        let manager = FileTransferManager::with_cleanup_options(
            dir.clone(),
            CleanupPolicy {
                idle: MAX_INCOMING_IDLE,
                interval: Duration::from_secs(3600),
            },
            spawn_cleanup,
        );
        manager
            .start_incoming_transfer(2, FileMetadata::new("partial.bin".into(), 2, sha256(b"ab")))
            .unwrap();
        let incoming = Arc::downgrade(&manager.incoming);
        let started = Instant::now();
        drop(manager);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(incoming.upgrade().is_none());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn failed_unlink_stays_charged_and_is_retried_without_receiver_traffic() {
        struct RestoreDirectory {
            output: PathBuf,
            moved: PathBuf,
        }
        impl RestoreDirectory {
            fn restore(&self) {
                if self.moved.exists() {
                    std::fs::remove_file(&self.output).unwrap();
                    std::fs::rename(&self.moved, &self.output).unwrap();
                }
            }
        }
        impl Drop for RestoreDirectory {
            fn drop(&mut self) {
                self.restore();
            }
        }
        let dir = temp_dir("worker-unlink-error");
        let output = dir.join("out");
        std::fs::create_dir(&output).unwrap();
        let manager = timed_manager(output.clone(), Duration::from_millis(25));
        for stream_id in [2, 3] {
            manager
                .start_incoming_transfer(
                    stream_id,
                    FileMetadata::new(format!("{stream_id}.bin"), MAX_INCOMING_FILE_SIZE, [0; 32]),
                )
                .unwrap();
        }
        let restore = RestoreDirectory {
            output: output.clone(),
            moved: dir.join("moved"),
        };
        {
            // A non-directory parent produces a real unlink error even as root.
            // Hold the state lock while changing the fixture namespace so the
            // worker cannot observe the intermediate missing-directory state.
            let _incoming = manager.incoming.lock().unwrap();
            std::fs::rename(&restore.output, &restore.moved).unwrap();
            std::fs::write(&restore.output, b"unlink obstruction").unwrap();
        }
        wait_until(|| {
            manager
                .incoming
                .lock()
                .unwrap()
                .values()
                .all(|transfer| transfer.cleanup_pending)
        });
        {
            let incoming = manager.incoming.lock().unwrap();
            assert_eq!(incoming.len(), 2);
            assert!(incoming
                .values()
                .all(|transfer| transfer.file.is_none() && transfer.cleanup_error.is_some()));
            assert_eq!(
                incoming
                    .values()
                    .map(|transfer| transfer.metadata.size)
                    .sum::<u64>(),
                MAX_RESERVED_INCOMING_BYTES
            );
        }
        let error = manager
            .start_incoming_transfer(4, FileMetadata::new("extra.bin".into(), 1, sha256(b"x")))
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("aggregate reserved storage limit"));
        assert!(manager
            .write_chunk(2, b"x")
            .unwrap_err()
            .to_string()
            .contains("awaiting temporary cleanup"));
        {
            let _incoming = manager.incoming.lock().unwrap();
            restore.restore();
        }
        // Restoring the filesystem is sufficient: no receiver call drives retry.
        wait_until(|| std::fs::read_dir(&output).unwrap().count() == 0);
        assert!(manager.incoming.lock().unwrap().is_empty());
        drop(manager);
        drop(restore);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn accepts_file_matching_announced_hash() {
        let dir = temp_dir("ok");
        let manager = FileTransferManager::new(dir.clone());
        let data = b"chunk-one|chunk-two";
        let meta = FileMetadata::new("ok.bin".into(), data.len() as u64, sha256(data));
        manager.start_incoming_transfer(2, meta).unwrap();
        manager.write_chunk(2, &data[..9]).unwrap();
        manager.write_chunk(2, &data[9..]).unwrap();
        assert_eq!(std::fs::read(dir.join("ok.bin")).unwrap(), data);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn publishes_only_after_complete_verified_transfer() {
        let dir = temp_dir("atomic");
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("verified.bin".into(), 6, sha256(b"abcdef"));
        manager.start_incoming_transfer(2, meta).unwrap();
        manager.write_chunk(2, b"abc").unwrap();
        assert!(!dir.join("verified.bin").exists());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        manager.write_chunk(2, b"def").unwrap();
        assert_eq!(std::fs::read(dir.join("verified.bin")).unwrap(), b"abcdef");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        drop(manager);
        assert!(dir.join("verified.bin").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn final_publication_never_overwrites_a_racing_file() {
        let dir = temp_dir("publish-race");
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("keep.bin".into(), 4, sha256(b"peer"));
        manager.start_incoming_transfer(2, meta).unwrap();
        std::fs::write(dir.join("keep.bin"), b"original").unwrap();
        assert!(manager.write_chunk(2, b"peer").is_err());
        assert_eq!(std::fs::read(dir.join("keep.bin")).unwrap(), b"original");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        assert!(manager.incoming.lock().unwrap().is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn dropping_manager_cleans_abandoned_partial_files() {
        let dir = temp_dir("drop");
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("abandoned.bin".into(), 6, sha256(b"abcdef"));
        manager.start_incoming_transfer(2, meta).unwrap();
        manager.write_chunk(2, b"abc").unwrap();
        drop(manager);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn bounds_active_transfers_before_creating_another_file() {
        let dir = temp_dir("quota");
        let manager = FileTransferManager::new(dir.clone());
        for index in 0..MAX_ACTIVE_INCOMING_TRANSFERS {
            let meta = FileMetadata::new(format!("{index}.bin"), 1, sha256(b"x"));
            manager
                .start_incoming_transfer(index as u32 + 2, meta)
                .unwrap();
        }
        let extra = FileMetadata::new("extra.bin".into(), 1, sha256(b"x"));
        assert!(manager.start_incoming_transfer(1000, extra).is_err());
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            MAX_ACTIVE_INCOMING_TRANSFERS
        );
        manager.write_chunk(2, b"x").unwrap();
        let next = FileMetadata::new("next.bin".into(), 1, sha256(b"x"));
        manager.start_incoming_transfer(1001, next).unwrap();
        drop(manager);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn admission_expiry_reclaims_slots_files_and_reserved_bytes() {
        let dir = temp_dir("expire-admission");
        let manager = FileTransferManager::new(dir.clone());
        let started = Instant::now();
        // Fill both the 64-transfer limit and the 2 GiB reservation limit.
        for index in 0..MAX_ACTIVE_INCOMING_TRANSFERS {
            let size = match index {
                0 => MAX_INCOMING_FILE_SIZE,
                1 => MAX_INCOMING_FILE_SIZE - (MAX_ACTIVE_INCOMING_TRANSFERS as u64 - 2),
                _ => 1,
            };
            let meta = FileMetadata::new(format!("{index}.bin"), size, [0; 32]);
            manager
                .start_incoming_transfer_with_clock(index as u32 + 2, meta, || started)
                .unwrap();
        }
        let expired_paths: Vec<_> = manager
            .incoming
            .lock()
            .unwrap()
            .values()
            .map(|transfer| transfer.temporary_path.clone())
            .collect();
        let next = FileMetadata::new("0.bin".into(), MAX_INCOMING_FILE_SIZE, [0; 32]);
        manager
            .start_incoming_transfer_with_clock(2, next, || started + MAX_INCOMING_IDLE)
            .unwrap();
        assert!(expired_paths.iter().all(|path| !path.exists()));
        assert_eq!(manager.incoming.lock().unwrap().len(), 1);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        drop(manager);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn expired_transfer_cannot_resume_on_a_chunk_operation() {
        let dir = temp_dir("expire-chunk");
        let manager = FileTransferManager::new(dir.clone());
        let started = Instant::now();
        let meta = FileMetadata::new("late.bin".into(), 2, sha256(b"ab"));
        manager
            .start_incoming_transfer_with_clock(2, meta, || started)
            .unwrap();
        assert!(manager
            .write_chunk_with_clock(2, b"a", || started + MAX_INCOMING_IDLE)
            .is_err());
        assert!(manager.incoming.lock().unwrap().is_empty());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn only_nonempty_byte_progress_refreshes_the_idle_deadline() {
        let dir = temp_dir("progress-deadline");
        let manager = FileTransferManager::new(dir.clone());
        let started = Instant::now();
        let meta = FileMetadata::new("progress.bin".into(), 3, sha256(b"abc"));
        manager
            .start_incoming_transfer_with_clock(2, meta, || started)
            .unwrap();
        let progress = started + MAX_INCOMING_IDLE - Duration::from_secs(1);
        manager
            .write_chunk_with_clock(2, b"a", || progress)
            .unwrap();
        manager
            .write_chunk_with_clock(2, b"", || {
                progress + MAX_INCOMING_IDLE - Duration::from_secs(1)
            })
            .unwrap();
        assert!(manager
            .write_chunk_with_clock(2, b"b", || progress + MAX_INCOMING_IDLE)
            .is_err());
        assert!(manager.incoming.lock().unwrap().is_empty());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn storage_limit_is_reserved_before_creation_and_released_after_write_error() {
        let dir = temp_dir("storage-quota-error");
        let manager = FileTransferManager::new(dir.clone());
        let now = Instant::now();
        for stream_id in [2, 3] {
            let meta =
                FileMetadata::new(format!("{stream_id}.bin"), MAX_INCOMING_FILE_SIZE, [0; 32]);
            manager
                .start_incoming_transfer_with_clock(stream_id, meta, || now)
                .unwrap();
        }
        let extra = FileMetadata::new("extra.bin".into(), 1, sha256(b"x"));
        assert!(manager
            .start_incoming_transfer_with_clock(4, extra, || now)
            .is_err());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
        {
            let mut incoming = manager.incoming.lock().unwrap();
            let transfer = incoming.get_mut(&2).unwrap();
            transfer.file = Some(File::open(&transfer.temporary_path).unwrap());
        }
        assert!(manager.write_chunk_with_clock(2, b"x", || now).is_err());
        let replacement =
            FileMetadata::new("replacement.bin".into(), MAX_INCOMING_FILE_SIZE, [0; 32]);
        manager
            .start_incoming_transfer_with_clock(4, replacement, || now)
            .unwrap();
        assert_eq!(manager.incoming.lock().unwrap().len(), 2);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
        drop(manager);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn completion_releases_active_reservation_but_keeps_the_verified_file() {
        let dir = temp_dir("storage-quota-complete");
        let manager = FileTransferManager::new(dir.clone());
        let now = Instant::now();
        for (stream_id, size) in [(2, MAX_INCOMING_FILE_SIZE), (3, MAX_INCOMING_FILE_SIZE - 1)] {
            let meta = FileMetadata::new(format!("{stream_id}.bin"), size, [0; 32]);
            manager
                .start_incoming_transfer_with_clock(stream_id, meta, || now)
                .unwrap();
        }
        let last_byte = FileMetadata::new("verified.bin".into(), 1, sha256(b"x"));
        manager
            .start_incoming_transfer_with_clock(4, last_byte, || now)
            .unwrap();
        manager.write_chunk_with_clock(4, b"x", || now).unwrap();
        let replacement = FileMetadata::new("next.bin".into(), 1, sha256(b"y"));
        manager
            .start_incoming_transfer_with_clock(5, replacement, || now)
            .unwrap();
        drop(manager);
        assert_eq!(std::fs::read(dir.join("verified.bin")).unwrap(), b"x");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn concurrent_admissions_cannot_exceed_the_storage_reservation_limit() {
        use std::sync::{Arc, Barrier};
        static WORKERS_STARTED: AtomicU32 = AtomicU32::new(0);
        fn counted_spawn(task: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
            WORKERS_STARTED.fetch_add(1, Ordering::SeqCst);
            spawn_cleanup(task)
        }
        WORKERS_STARTED.store(0, Ordering::SeqCst);
        let dir = temp_dir("concurrent-storage-quota");
        let manager = Arc::new(FileTransferManager::with_cleanup_options(
            dir.clone(),
            CleanupPolicy {
                idle: MAX_INCOMING_IDLE,
                interval: CLEANUP_INTERVAL,
            },
            counted_spawn,
        ));
        let barrier = Arc::new(Barrier::new(16));
        let size = MAX_INCOMING_FILE_SIZE / 4;
        let threads: Vec<_> = (0..16)
            .map(|index| {
                let manager = Arc::clone(&manager);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    let meta = FileMetadata::new(format!("{index}.bin"), size, [0; 32]);
                    manager.start_incoming_transfer(index + 2, meta).is_ok()
                })
            })
            .collect();
        let admitted = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .filter(|ok| *ok)
            .count();
        assert_eq!(admitted as u64 * size, MAX_RESERVED_INCOMING_BYTES);
        assert_eq!(WORKERS_STARTED.load(Ordering::SeqCst), 1);
        let incoming = manager.incoming.lock().unwrap();
        assert_eq!(
            incoming
                .values()
                .map(|transfer| transfer.metadata.size)
                .sum::<u64>(),
            MAX_RESERVED_INCOMING_BYTES
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), admitted);
        drop(incoming);
        drop(manager);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn write_failure_discards_file_and_releases_admission() {
        let dir = temp_dir("write-error");
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("failed.bin".into(), 1, sha256(b"x"));
        manager.start_incoming_transfer(2, meta.clone()).unwrap();
        {
            let mut incoming = manager.incoming.lock().unwrap();
            let transfer = incoming.get_mut(&2).unwrap();
            // A read-only descriptor deterministically rejects the attempted write.
            transfer.file = Some(File::open(&transfer.temporary_path).unwrap());
        }
        assert!(manager.write_chunk(2, b"x").is_err());
        assert!(manager.incoming.lock().unwrap().is_empty());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        manager.start_incoming_transfer(2, meta).unwrap();
        drop(manager);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn sync_failure_cannot_publish_or_leave_a_partial_file() {
        let dir = temp_dir("sync-error");
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("failed.bin".into(), 1, sha256(b"x"));
        manager.start_incoming_transfer(2, meta).unwrap();
        {
            let mut incoming = manager.incoming.lock().unwrap();
            // /dev/null accepts writes but rejects fsync on supported Unix targets.
            incoming.get_mut(&2).unwrap().file =
                Some(OpenOptions::new().write(true).open("/dev/null").unwrap());
        }
        let error = manager.write_chunk(2, b"x").unwrap_err();
        assert!(error.to_string().contains("Failed to flush received file"));
        assert!(manager.incoming.lock().unwrap().is_empty());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_and_removes_file_with_hash_mismatch() {
        let dir = temp_dir("bad");
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("bad.bin".into(), 4, sha256(b"good"));
        manager.start_incoming_transfer(2, meta).unwrap();
        let err = manager.write_chunk(2, b"evil").unwrap_err();
        assert!(err.to_string().contains("SHA-256 mismatch"));
        assert!(!dir.join("bad.bin").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_data_beyond_announced_size() {
        let dir = temp_dir("long");
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("long.bin".into(), 2, sha256(b"ab"));
        manager.start_incoming_transfer(2, meta).unwrap();
        assert!(manager.write_chunk(2, b"abc").is_err());
        assert!(!dir.join("long.bin").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn never_overwrites_existing_file() {
        let dir = temp_dir("exists");
        std::fs::write(dir.join("keep.txt"), b"original").unwrap();
        let manager = FileTransferManager::new(dir.clone());
        let meta = FileMetadata::new("keep.txt".into(), 4, sha256(b"evil"));
        assert!(manager.start_incoming_transfer(2, meta).is_err());
        assert!(manager.write_chunk(2, b"evil").is_err());
        assert_eq!(std::fs::read(dir.join("keep.txt")).unwrap(), b"original");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn never_follows_symlinks() {
        let dir = temp_dir("link");
        let out = dir.join("out");
        std::fs::create_dir(&out).unwrap();
        let victim = dir.join("victim.txt");
        std::fs::write(&victim, b"original").unwrap();
        std::os::unix::fs::symlink(&victim, out.join("live.txt")).unwrap();
        let dangling = dir.join("created-through-link.txt");
        std::os::unix::fs::symlink(&dangling, out.join("dangling.txt")).unwrap();

        let manager = FileTransferManager::new(out);
        for (id, name) in [(2, "live.txt"), (3, "dangling.txt")] {
            let meta = FileMetadata::new(name.into(), 4, sha256(b"evil"));
            assert!(
                manager.start_incoming_transfer(id, meta).is_err(),
                "followed {name}"
            );
        }
        assert_eq!(std::fs::read(&victim).unwrap(), b"original");
        assert!(!dangling.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn zero_byte_transfer_is_verified_immediately() {
        let dir = temp_dir("empty");
        let manager = FileTransferManager::new(dir.clone());
        let good = FileMetadata::new("empty.bin".into(), 0, sha256(b""));
        manager.start_incoming_transfer(2, good).unwrap();
        assert_eq!(std::fs::read(dir.join("empty.bin")).unwrap(), b"");
        // Completed transfers do not stay registered.
        assert!(manager.write_chunk(2, b"").is_err());

        let bad = FileMetadata::new("forged.bin".into(), 0, sha256(b"not empty"));
        let err = manager.start_incoming_transfer(3, bad).unwrap_err();
        assert!(err.to_string().contains("SHA-256 mismatch"));
        assert!(!dir.join("forged.bin").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_oversized_announcement_before_writing() {
        let dir = temp_dir("huge");
        let manager = FileTransferManager::new(dir.clone());
        for size in [MAX_INCOMING_FILE_SIZE + 1, u64::MAX] {
            let meta = FileMetadata::new("huge.bin".into(), size, [0u8; 32]);
            assert!(manager.start_incoming_transfer(2, meta).is_err());
        }
        assert!(!dir.join("huge.bin").exists());
        let at_limit = FileMetadata::new("limit.bin".into(), MAX_INCOMING_FILE_SIZE, [0u8; 32]);
        manager.start_incoming_transfer(2, at_limit).unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn duplicate_stream_id_does_not_replace_active_transfer() {
        let dir = temp_dir("dup");
        let manager = FileTransferManager::new(dir.clone());
        let data = b"first";
        let first = FileMetadata::new("first.bin".into(), 5, sha256(data));
        manager.start_incoming_transfer(2, first).unwrap();
        let second = FileMetadata::new("second.bin".into(), 5, sha256(b"other"));
        assert!(manager.start_incoming_transfer(2, second).is_err());
        assert!(!dir.join("second.bin").exists());
        manager.write_chunk(2, data).unwrap();
        assert_eq!(std::fs::read(dir.join("first.bin")).unwrap(), data);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_path_traversal_filenames() {
        let dir = temp_dir("path");
        let manager = FileTransferManager::new(dir.join("out"));
        for name in ["../escape.bin", "/tmp/abs.bin", "a/b.bin", "..", ""] {
            let meta = FileMetadata::new(name.into(), 1, [0u8; 32]);
            assert!(
                manager.start_incoming_transfer(2, meta).is_err(),
                "accepted {name:?}"
            );
        }
        assert!(!dir.join("escape.bin").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
