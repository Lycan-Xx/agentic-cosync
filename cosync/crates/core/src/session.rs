use crate::envelope::{pairing_request_envelope};
use crate::error::{CosyncError, Result};
use crate::file_transfer::FileTransferManager;
use crate::hlc::{HlcTimestamp, HybridLogicalClock};
use crate::notification::NotificationMirror;
use crate::state::ConnectionState;
use crate::storage::Storage;
use crate::transport::{accept_connection, create_client_endpoint, create_server_endpoint, recv_envelope, send_envelope, PinnedCertVerifier};
use crate::wire::cosync::envelope::Payload;
use crate::wire::cosync::Envelope;
use quinn::Connection;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};

#[derive(Debug, Clone)]
pub enum SessionEvent {
    StateChanged(ConnectionState),
    ClipboardReceived { content: Vec<u8>, content_type: String, source: String },
    FileIncoming { transfer_id: String, file_name: String, file_size: u64 },
    FileProgress { transfer_id: String, chunk_index: u32, total_chunks: u32 },
    FileComplete { transfer_id: String, success: bool, path: String },
    NotificationReceived { package_name: String, title: String, text: String },
    PeerPaired { device_name: String, fingerprint: String },
    Error(String),
}

pub struct SessionManager {
    device_id: String,
    device_name: String,
    cert_der: Vec<u8>,
    key_der: Vec<u8>,
    hlc: Arc<Mutex<HybridLogicalClock>>,
    storage: Arc<Storage>,
    state_tx: watch::Sender<ConnectionState>,
    state_rx: watch::Receiver<ConnectionState>,
    event_tx: mpsc::Sender<SessionEvent>,
    event_rx: Mutex<Option<mpsc::Receiver<SessionEvent>>>,
    connections: Arc<Mutex<HashMap<String, Connection>>>,
    cert_verifier: Arc<PinnedCertVerifier>,
    server_endpoint: Arc<Mutex<Option<quinn::Endpoint>>>,
    file_transfer: Arc<FileTransferManager>,
    _notification_mirror: Arc<NotificationMirror>,
    shutdown: Arc<tokio::sync::Notify>,
}

impl SessionManager {
    pub fn new(device_name: String, cert_der: Vec<u8>, key_der: Vec<u8>, storage: Arc<Storage>) -> Result<Self> {
        let fingerprint = {
            let parsed = x509_parser::parse_x509_certificate(&cert_der)
                .map_err(|e| CosyncError::Cert(format!("Parse cert: {}", e)))?;
            let mut h = Sha256::new();
            h.update(parsed.1.public_key().raw);
            hex::encode(h.finalize())
        };
        let (state_tx, state_rx) = watch::channel(ConnectionState::Idle);
        let (event_tx, event_rx) = mpsc::channel(256);
        let hlc = Arc::new(Mutex::new(HybridLogicalClock::new(fingerprint.clone())));
        Ok(Self {
            device_id: fingerprint, device_name, cert_der, key_der, hlc, storage,
            state_tx, state_rx, event_tx, event_rx: Mutex::new(Some(event_rx)),
            connections: Arc::new(Mutex::new(HashMap::new())),
            cert_verifier: Arc::new(PinnedCertVerifier::new()),
            server_endpoint: Arc::new(Mutex::new(None)),
            file_transfer: Arc::new(FileTransferManager::new()),
            _notification_mirror: Arc::new(NotificationMirror::new()),
            shutdown: Arc::new(tokio::sync::Notify::new()),
        })
    }

    pub fn state_receiver(&self) -> watch::Receiver<ConnectionState> { self.state_rx.clone() }

    /// Takes the event receiver. Can only be called once; returns `None` on subsequent calls.
    /// Consumers (e.g. Tauri backend) use this to forward `SessionEvent`s to the frontend.
    pub async fn take_event_receiver(&self) -> Option<mpsc::Receiver<SessionEvent>> {
        self.event_rx.lock().await.take()
    }

    /// Returns this device's fingerprint (SHA-256 of the public key).
    pub fn device_id(&self) -> &str { &self.device_id }

    /// Returns a clone of the HLC handle for creating timestamps.
    pub fn hlc(&self) -> Arc<Mutex<HybridLogicalClock>> { self.hlc.clone() }

    /// Returns a clone of the storage handle for querying DB.
    pub fn storage(&self) -> Arc<Storage> { self.storage.clone() }

    /// Returns a clone of the file transfer manager.
    pub fn file_transfer(&self) -> Arc<FileTransferManager> { self.file_transfer.clone() }

    /// Returns the list of connected peer fingerprints.
    pub async fn connected_peers(&self) -> Vec<String> {
        self.connections.lock().await.keys().cloned().collect()
    }

    pub async fn start_server(&self, bind_addr: SocketAddr) -> Result<()> {
        let endpoint = create_server_endpoint(bind_addr, self.cert_der.clone(), self.key_der.clone())?;
        *self.server_endpoint.lock().await = Some(endpoint.clone());
        let hlc = self.hlc.clone();
        let event_tx = self.event_tx.clone();
        let device_id = self.device_id.clone();
        let storage = self.storage.clone();
        let file_transfer = self.file_transfer.clone();
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = accept_connection(&endpoint) => {
                        match result {
                            Ok(conn) => {
                                let addr = conn.remote_address();
                                tracing::info!(%addr, "Accepted connection");
                                let c = conn.clone(); let h = hlc.clone(); let et = event_tx.clone();
                                let did = device_id.clone(); let st = storage.clone(); let ft = file_transfer.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = Self::recv_loop(c, h, et, did, st, ft).await {
                                        tracing::error!("Recv loop error: {}", e);
                                    }
                                });
                            }
                            Err(e) => { tracing::error!("Accept error: {}", e); break; }
                        }
                    }
                    _ = shutdown.notified() => { break; }
                }
            }
        });
        Ok(())
    }

    async fn recv_loop(conn: Connection, hlc: Arc<Mutex<HybridLogicalClock>>,
        event_tx: mpsc::Sender<SessionEvent>, device_id: String,
        storage: Arc<Storage>, _file_transfer: Arc<FileTransferManager>) -> Result<()> {
        loop {
            match conn.accept_bi().await {
                Ok((mut _send, mut recv)) => {
                    match recv_envelope(&mut recv).await {
                        Ok(envelope) => {
                            let hlc_ts: HlcTimestamp = serde_json::from_slice(&envelope.hlc_timestamp)
                                .unwrap_or(HlcTimestamp::new(0, 0, "unknown".into()));
                            let mut hlc_lock = hlc.lock().await;
                            if let Err(true) = hlc_lock.receive(&hlc_ts) { continue; }
                            drop(hlc_lock);
                            Self::handle_envelope(&envelope, &event_tx, &device_id, &storage).await;
                        }
                        Err(e) => tracing::warn!("Recv error: {}", e),
                    }
                }
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => break,
                Err(e) => { tracing::warn!("Conn error: {}", e); break; }
            }
        }
        Ok(())
    }

    async fn handle_envelope(envelope: &Envelope, event_tx: &mpsc::Sender<SessionEvent>,
        _device_id: &str, storage: &Storage) {
        match &envelope.payload {
            Some(Payload::ClipboardUpdate(cb)) => {
                let _ = event_tx.send(SessionEvent::ClipboardReceived {
                    content: cb.content.clone(), content_type: cb.content_type.clone(),
                    source: hex::encode(&envelope.sender_device_id) }).await;
                let hlc_str = String::from_utf8_lossy(&envelope.hlc_timestamp).to_string();
                let _ = storage.insert_clipboard(&cb.content, &cb.content_type,
                    Some(&hex::encode(&envelope.sender_device_id)), Some(&hlc_str));
            }
            Some(Payload::FileMeta(fm)) => {
                let _ = event_tx.send(SessionEvent::FileIncoming {
                    transfer_id: fm.transfer_id.clone(), file_name: fm.file_name.clone(),
                    file_size: fm.file_size }).await;
            }
            Some(Payload::FileChunk(fc)) => {
                let _ = event_tx.send(SessionEvent::FileProgress {
                    transfer_id: fc.transfer_id.clone(), chunk_index: fc.chunk_index, total_chunks: 0 }).await;
            }
            Some(Payload::FileAck(fa)) => {
                let _ = event_tx.send(SessionEvent::FileComplete {
                    transfer_id: fa.transfer_id.clone(), success: fa.success, path: String::new() }).await;
            }
            Some(Payload::NotificationEvent(ne)) => {
                let _ = event_tx.send(SessionEvent::NotificationReceived {
                    package_name: ne.package_name.clone(), title: ne.title.clone(),
                    text: ne.text.clone() }).await;
            }
            Some(Payload::Heartbeat(_)) => {}
            Some(Payload::PairingRequest(pr)) => {
                tracing::info!(device = %pr.device_name, "Pairing request received");
                let _ = event_tx.send(SessionEvent::PeerPaired {
                    device_name: pr.device_name.clone(), fingerprint: pr.public_key_fingerprint.clone() }).await;
                let _ = storage.upsert_device(&hex::encode(&envelope.sender_device_id),
                    &pr.device_name, &pr.public_key_fingerprint, None);
            }
            Some(Payload::PairingAck(pa)) => {
                if pa.accepted {
                    tracing::info!(device = %pa.device_name, "Pairing accepted");
                    let _ = event_tx.send(SessionEvent::PeerPaired {
                        device_name: pa.device_name.clone(), fingerprint: pa.public_key_fingerprint.clone() }).await;
                } else {
                    tracing::warn!(reason = %pa.reason, "Pairing rejected");
                }
            }
            _ => {}
        }
    }

    pub async fn pair_with_peer(&self, peer_addr: SocketAddr, peer_fingerprint: &str) -> Result<()> {
        self.cert_verifier.add_fingerprint(peer_fingerprint);
        let client_endpoint = create_client_endpoint(self.cert_verifier.clone())?;
        let connect = client_endpoint.connect(peer_addr, "cosync-peer").unwrap();
        let connection = connect.await.map_err(|e| CosyncError::Transport(format!("Connect: {}", e)))?;
        let mut hlc = self.hlc.lock().await;
        let ts = hlc.now();
        let ts_bytes = serde_json::to_vec(&ts)?;
        drop(hlc);
        let token = rand::random::<[u8; 32]>();
        let envelope = pairing_request_envelope(self.device_id.as_bytes(), &ts_bytes,
            &self.device_name, &self.device_id, peer_addr.port() as u32, &token);
        let (mut send, mut recv) = connection.open_bi().await
            .map_err(|e| CosyncError::Transport(e.to_string()))?;
        send_envelope(&mut send, &envelope).await?;
        let ack = recv_envelope(&mut recv).await?;
        if let Some(Payload::PairingAck(pa)) = ack.payload {
            if pa.accepted {
                self.storage.upsert_device(&hex::encode(&ack.sender_device_id),
                    &pa.device_name, &pa.public_key_fingerprint,
                    Some(&peer_addr.ip().to_string()))?;
                self.cert_verifier.add_fingerprint(&pa.public_key_fingerprint);
                self.connections.lock().await.insert(pa.public_key_fingerprint.clone(), connection.clone());
                let name = pa.device_name.clone();
                let _ = self.state_tx.send(ConnectionState::Connected(name));
                Ok(())
            } else {
                Err(CosyncError::Pairing(format!("Rejected: {}", pa.reason)))
            }
        } else {
            Err(CosyncError::Pairing("Expected PairingAck".into()))
        }
    }

    pub async fn send_to(&self, fingerprint: &str, envelope: &Envelope) -> Result<()> {
        let connections = self.connections.lock().await;
        if let Some(conn) = connections.get(fingerprint) {
            let (mut send, _) = conn.open_bi().await.map_err(|e| CosyncError::Transport(e.to_string()))?;
            send_envelope(&mut send, envelope).await
        } else {
            Err(CosyncError::DeviceNotFound(fingerprint.into()))
        }
    }

    pub async fn broadcast(&self, envelope: &Envelope) -> Result<()> {
        let connections = self.connections.lock().await;
        for (fp, conn) in connections.iter() {
            if let Ok((mut send, _)) = conn.open_bi().await {
                if let Err(e) = send_envelope(&mut send, envelope).await {
                    tracing::warn!(peer = %fp, "Broadcast error: {}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn shutdown(&self) { self.shutdown.notify_waiters(); }
}