use cosync_core::{
    ConnectionState, DeviceIdentity, DiscoveryService, SessionEvent, SessionManager, Storage,
};
use ed25519_dalek::pkcs8::EncodePrivateKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::state::CosyncState;

// ── Data types shared between Rust and the frontend ──────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_name: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDeviceView {
    pub device_id: String,
    pub device_name: String,
    pub fingerprint: String,
    pub last_known_ip: Option<String>,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeerView {
    pub device_name: String,
    pub fingerprint: String,
    pub addresses: Vec<String>,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FrontendEvent {
    ConnectionStateChanged { state: String },
    DeviceFound(DiscoveredPeerView),
    DeviceLost { device_name: String },
    PairingRequest { device_name: String, fingerprint: String },
    ClipboardReceived { content: String, source: String },
    FileIncoming { transfer_id: String, file_name: String, file_size: u64 },
    FileProgress { transfer_id: String, chunk_index: u32, total_chunks: u32 },
    FileComplete { transfer_id: String, success: bool, path: String },
    NotificationReceived { package_name: String, title: String, text: String },
    Error { message: String },
}

// ── Tauri commands ────────────────────────────────────────────────────

/// Returns this device's identity info (name + fingerprint).
#[tauri::command]
pub async fn get_device_info(state: State<'_, CosyncState>) -> Result<DeviceInfo, String> {
    let identity = state.identity.lock().await;
    Ok(DeviceInfo {
        device_name: identity.device_name.clone(),
        fingerprint: identity.fingerprint().map_err(|e| e.to_string())?,
    })
}

/// Returns the current connection state.
#[tauri::command]
pub async fn get_connection_state(state: State<'_, CosyncState>) -> Result<String, String> {
    let conn = state.connection_state.lock().await;
    Ok(format!("{:?}", conn))
}

/// Starts mDNS discovery + QUIC server. Emits events as devices are found.
#[tauri::command]
pub async fn start_discovery(
    app: AppHandle,
    state: State<'_, CosyncState>,
) -> Result<(), String> {
    let identity = state.identity.lock().await;
    let device_name = identity.device_name.clone();
    let cert_der = identity.cert_der.clone();
    let key_der = identity
        .signing_key
        .to_pkcs8_der()
        .map_err(|e| format!("PKCS#8 export: {}", e))?
        .as_bytes()
        .to_vec();
    let fingerprint = identity.fingerprint().map_err(|e| e.to_string())?;
    drop(identity);

    // Create storage
    let storage = {
        let app_data = state.app_data_dir.lock().await.clone();
        std::fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
        Arc::new(Storage::open(&app_data.join("cosync.db")).map_err(|e| e.to_string())?)
    };

    // Create session manager
    let session = SessionManager::new(
        device_name.clone(),
        cert_der,
        key_der,
        storage,
    )
    .map_err(|e| e.to_string())?;

    // Start QUIC server on a random port
    let bind_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    session.start_server(bind_addr).map_err(|e| e.to_string())?;

    // Create discovery service and advertise
    let discovery = DiscoveryService::new().map_err(|e| e.to_string())?;
    discovery
        .advertise(&device_name, 0, &fingerprint)
        .map_err(|e| e.to_string())?;

    // Store in state
    {
        let mut sm = state.session_manager.lock().await;
        *sm = Some(session);
    }
    {
        let mut disc = state.discovery.lock().await;
        *disc = Some(discovery);
    }
    {
        let mut conn = state.connection_state.lock().await;
        *conn = ConnectionState::Discovering;
        let _ = app.emit(
            "cosync://event",
            FrontendEvent::ConnectionStateChanged {
                state: "Discovering".into(),
            },
        );
    }

    // Spawn a task that browses for peers and forwards events to the frontend
    let app_handle = app.clone();
    let disc_clone = state.discovery.clone();
    let conn_state = state.connection_state.clone();
    tokio::spawn(async move {
        let disc_guard = disc_clone.lock().await;
        if let Some(ref discovery) = *disc_guard {
            match discovery.browse() {
                Ok(mut peer_rx) => {
                    drop(disc_guard);
                    while let Some(peer) = peer_rx.recv().await {
                        let view = DiscoveredPeerView {
                            device_name: peer.device_name.clone(),
                            fingerprint: peer.fingerprint.clone(),
                            addresses: peer.addresses.iter().map(|a| a.to_string()).collect(),
                            port: peer.port,
                        };
                        let _ = app_handle.emit("cosync://event", FrontendEvent::DeviceFound(view));
                    }
                }
                Err(e) => {
                    let mut conn = conn_state.lock().await;
                    *conn = ConnectionState::Error(e.to_string());
                    let _ = app_handle.emit(
                        "cosync://event",
                        FrontendEvent::Error { message: e.to_string() },
                    );
                }
            }
        }
    });

    Ok(())
}

/// Stops the current discovery/session.
#[tauri::command]
pub async fn stop_discovery(state: State<'_, CosyncState>) -> Result<(), String> {
    let mut sm = state.session_manager.lock().await;
    if let Some(ref session) = *sm {
        session.shutdown().await;
    }
    *sm = None;

    let mut disc = state.discovery.lock().await;
    if let Some(ref discovery) = *disc {
        discovery.shutdown().map_err(|e| e.to_string())?;
    }
    *disc = None;

    let mut conn = state.connection_state.lock().await;
    *conn = ConnectionState::Idle;
    Ok(())
}

/// Pairs with a discovered device given its IP address and port.
#[tauri::command]
pub async fn pair_with_device(
    app: AppHandle,
    state: State<'_, CosyncState>,
    peer_ip: String,
    peer_port: u16,
    peer_fingerprint: String,
) -> Result<(), String> {
    let addr: std::net::SocketAddr = format!("{}:{}", peer_ip, peer_port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    {
        let mut conn = state.connection_state.lock().await;
        *conn = ConnectionState::Pairing;
        let _ = app.emit(
            "cosync://event",
            FrontendEvent::ConnectionStateChanged {
                state: "Pairing".into(),
            },
        );
    }

    let sm = state.session_manager.lock().await;
    let session = sm.as_ref().ok_or("No active session — call start_discovery first")?;
    session
        .pair_with_peer(addr, &peer_fingerprint)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut conn = state.connection_state.lock().await;
        *conn = ConnectionState::Connected(String::new());
        let _ = app.emit(
            "cosync://event",
            FrontendEvent::ConnectionStateChanged {
                state: "Connected".into(),
            },
        );
    }

    Ok(())
}

/// Unpairs a device by its ID.
#[tauri::command]
pub async fn unpair_device(
    state: State<'_, CosyncState>,
    device_id: String,
) -> Result<(), String> {
    let sm = state.session_manager.lock().await;
    // For now, just remove from storage. The session manager doesn't have a direct unpair method.
    // We'll add storage-backed unpair in a follow-up.
    drop(sm);

    let mut conn = state.connection_state.lock().await;
    *conn = ConnectionState::Idle;
    Ok(())
}

/// Returns the list of currently paired devices from storage.
#[tauri::command]
pub async fn get_paired_devices(state: State<'_, CosyncState>) -> Result<Vec<PairedDeviceView>, String> {
    let sm = state.session_manager.lock().await;
    let session = sm.as_ref().ok_or("No active session")?;
    // SessionManager doesn't expose storage directly; we'll use our own storage ref.
    // For the scaffold, return an empty list — will wire up in next iteration.
    drop(sm);

    Ok(vec![])
}

/// Retrieves the local clipboard history.
#[tauri::command]
pub async fn get_clipboard_history(state: State<'_, CosyncState>) -> Result<Vec<String>, String> {
    // Will be wired to Storage::get_clipboard_history in next iteration.
    Ok(vec![])
}

/// Returns this device's fingerprint.
#[tauri::command]
pub async fn get_device_fingerprint(state: State<'_, CosyncState>) -> Result<String, String> {
    let identity = state.identity.lock().await;
    identity.fingerprint().map_err(|e| e.to_string())
}