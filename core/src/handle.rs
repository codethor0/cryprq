// © 2025 Thor Thor
// SPDX-License-Identifier: MIT

use crate::error::CrypRqErrorCode;
use anyhow::Context;
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tokio::runtime::Runtime;

enum ConnectionState {
    Listener(tokio::task::JoinHandle<()>),
    Dialer(tokio::task::JoinHandle<()>),
}

pub struct CrypRqHandle {
    pub(crate) runtime: Runtime,
    pub(crate) allow_peers: Vec<String>,
    connection: Mutex<Option<ConnectionState>>,
}

impl CrypRqHandle {
    pub fn new(allow_peers: Vec<String>) -> Result<Self, CrypRqErrorCode> {
        static LOGGER: OnceCell<()> = OnceCell::new();
        LOGGER.get_or_init(|| {
            let _ =
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                    .try_init();
        });

        let runtime = Runtime::new().map_err(|_| CrypRqErrorCode::CRYPRQ_ERR_RUNTIME)?;
        Ok(Self {
            runtime,
            allow_peers,
            connection: Mutex::new(None),
        })
    }

    fn set_connection(&self, state: ConnectionState) -> Result<(), CrypRqErrorCode> {
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| CrypRqErrorCode::CRYPRQ_ERR_INTERNAL)?;
        if guard.is_some() {
            return Err(CrypRqErrorCode::CRYPRQ_ERR_ALREADY_CONNECTED);
        }
        *guard = Some(state);
        Ok(())
    }

    pub fn clear_connection(&self) {
        if let Ok(mut guard) = self.connection.lock() {
            if let Some(state) = guard.take() {
                match state {
                    ConnectionState::Listener(handle) => handle.abort(),
                    ConnectionState::Dialer(handle) => handle.abort(),
                }
            }
        }
    }

    pub fn spawn_listener(&self, addr: String) -> Result<(), CrypRqErrorCode> {
        let allow = self.allow_peers.clone();
        let handle = self.runtime.spawn(async move {
            if let Err(err) = async {
                if !allow.is_empty() {
                    p2p::set_allowed_peers(&allow)
                        .await
                        .context("failed to set allowlist")?;
                }
                p2p::start_listener(&addr)
                    .await
                    .context("listener task failed")?;
                Ok::<(), anyhow::Error>(())
            }
            .await
            {
                log::error!("listener task exited: {err:?}");
            }
        });
        self.set_connection(ConnectionState::Listener(handle))
    }

    pub fn spawn_dialer(&self, addr: String) -> Result<(), CrypRqErrorCode> {
        let allow = self.allow_peers.clone();
        let handle = self.runtime.spawn(async move {
            if let Err(err) = async {
                if !allow.is_empty() {
                    p2p::set_allowed_peers(&allow)
                        .await
                        .context("failed to set allowlist")?;
                }
                p2p::dial_peer(addr).await.context("dial task failed")?;
                Ok::<(), anyhow::Error>(())
            }
            .await
            {
                log::error!("dial task exited: {err:?}");
            }
        });
        self.set_connection(ConnectionState::Dialer(handle))
    }
}

impl Drop for CrypRqHandle {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.connection.lock() {
            if let Some(state) = guard.take() {
                match state {
                    ConnectionState::Listener(handle) => handle.abort(),
                    ConnectionState::Dialer(handle) => handle.abort(),
                }
            }
        }
    }
}
