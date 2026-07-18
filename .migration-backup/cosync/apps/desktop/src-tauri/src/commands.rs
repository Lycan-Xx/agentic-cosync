use cosync_core::{
    ConnectionState, DeviceIdentity, DiscoveryService, HlcTimestamp, SessionEvent,
    SessionManager, Storage,
};
use ed25519_dalek::pkcs8::EncodePrivateKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::state::{session_event_to_frontend, CosyncState};

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
pub struct ClipboardEntryView {
    pub id: i64,
    pub content: String,
    pub content_type: String,
    pub source_device_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFileResult {
    pub transfer_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub total_chunks: u32,
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

    // Reuse the shared storage (already opened in CosyncState::new)
    let storage = state.storage.clone();

    // Create session manager
    let session = SessionManager::new(device_name.clone(), cert_der, key_der, storage.clone())
        .map_err(|e| e.to_string())?;

    // Start QUIC server on a random port
    let bind_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    session.start_server(bind_addr).map_err(|e| e.to_string())?;

    // Take the event receiver BEFORE storing the session, so we can forward events
    let event_rx = session
        .take_event_receiver()
        .await
        .ok_or("Event receiver already taken")?;

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

    // Spawn event forwarder: reads SessionEvents and emits them to the frontend
    let app_handle = app.clone();
    tokio::spawn(async move {
        while let Some(session_event) = event_rx.recv().await {
            let frontend_event = session_event_to_frontend(session_event).await;
            let _ = app_handle.emit("cosync://event", frontend_event);
        }
    });

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
                        let _ =
                            app_handle.emit("cosync://event", FrontendEvent::DeviceFound(view));
                    }
                }
                Err(e) => {
                    let mut conn = conn_state.lock().await;
                    *conn = ConnectionState::Error(e.to_string());
                    let _ = app_handle.emit(
                        "cosync://event",
                        FrontendEvent::Error {
                            message: e.to_string(),
                        },
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
    let session = sm
        .as_ref()
        .ok_or("No active session — call start_discovery first")?;
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
    state
        .storage
        .remove_device(&device_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns the list of paired devices from storage.
#[tauri::command]
pub async fn get_paired_devices(
    state: State<'_, CosyncState>,
) -> Result<Vec<PairedDeviceView>, String> {
    let devices = state
        .storage
        .get_devices()
        .map_err(|e| e.to_string())?;
    Ok(devices
        .into_iter()
        .map(|d| PairedDeviceView {
            device_id: d.device_id,
            device_name: d.device_name,
            fingerprint: d.fingerprint,
            last_known_ip: d.last_known_ip,
            last_seen_at: d.last_seen_at,
        })
        .collect())
}

/// Retrieves the local clipboard history from SQLite.
#[tauri::command]
pub async fn get_clipboard_history(
    state: State<'_, CosyncState>,
) -> Result<Vec<ClipboardEntryView>, String> {
    let entries = state
        .storage
        .get_clipboard_history(100)
        .map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| ClipboardEntryView {
            id: e.id,
            content: String::from_utf8_lossy(&e.content).to_string(),
            content_type: e.content_type,
            source_device_id: e.source_device_id,
            created_at: e.created_at,
        })
        .collect())
}

/// Clears all clipboard history from SQLite.
#[tauri::command]
pub async fn clear_clipboard_history(state: State<'_, CosyncState>) -> Result<(), String> {
    state
        .storage
        .clear_clipboard_history()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Sends a text clipboard entry to all connected peers.
#[tauri::command]
pub async fn send_clipboard(
    state: State<'_, CosyncState>,
    content: String,
) -> Result<(), String> {
    let sm = state.session_manager.lock().await;
    let session = sm
        .as_ref()
        .ok_or("No active session — start discovery and pair first")?;

    // Check we have at least one connected peer
    let peers = session.connected_peers().await;
    if peers.is_empty() {
        return Err("No connected peers".into());
    }

    // Build HLC timestamp
    let hlc = session.hlc();
    let mut hlc_lock = hlc.lock().await;
    let ts = hlc_lock.now();
    let ts_bytes = serde_json::to_vec(&ts).map_err(|e| format!("Serialize HLC: {}", e))?;
    drop(hlc_lock);

    // Create and broadcast the clipboard envelope
    let device_id_bytes = hex::decode(session.device_id())
        .map_err(|e| format!("Decode device_id: {}", e))?;
    let envelope = cosync_core::envelope::clipboard_envelope(
        &device_id_bytes,
        &ts_bytes,
        content.as_bytes(),
        "text/plain",
    );

    // Store locally
    state
        .storage
        .insert_clipboard(content.as_bytes(), "text/plain", None, None)
        .map_err(|e| e.to_string())?;

    session
        .broadcast(&envelope)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Sends a file to all connected peers.
#[tauri::command]
pub async fn send_file(
    state: State<'_, CosyncState>,
    file_path: String,
) -> Result<SendFileResult, String> {
    let sm = state.session_manager.lock().await;
    let session = sm
        .as_ref()
        .ok_or("No active session — start discovery and pair first")?;

    let peers = session.connected_peers().await;
    if peers.is_empty() {
        return Err("No connected peers".into());
    }

    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Generate a unique transfer ID
    let transfer_id = format!(
        "tx-{}-{}",
        hex::encode(&session.device_id().as_bytes()[..4]),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    // Read and chunk the file
    let ft = session.file_transfer();
    let (info, chunks) = ft
        .send_file(path, &transfer_id)
        .await
        .map_err(|e| e.to_string())?;

    // Build HLC timestamp for file meta envelope
    let hlc = session.hlc();
    let mut hlc_lock = hlc.lock().await;
    let ts = hlc_lock.now();
    let ts_bytes = serde_json::to_vec(&ts).map_err(|e| format!("Serialize HLC: {}", e))?;
    drop(hlc_lock);

    let device_id_bytes = hex::decode(session.device_id())
        .map_err(|e| format!("Decode device_id: {}", e))?;

    // Send file meta first
    let meta_envelope = cosync_core::envelope::file_meta_envelope(
        &device_id_bytes,
        &ts_bytes,
        &info.file_name,
        info.file_size,
        &info.sha256,
        "application/octet-stream",
        info.total_chunks,
        &info.transfer_id,
    );
    session
        .broadcast(&meta_envelope)
        .await
        .map_err(|e| e.to_string())?;

    // Send chunks (with a small delay between each to avoid overwhelming the connection)
    for (i, chunk) in chunks.iter().enumerate() {
        let hlc = session.hlc();
        let mut hlc_lock = hlc.lock().await;
        let ts = hlc_lock.now();
        let ts_bytes = serde_json::to_vec(&ts).map_err(|e| format!("Serialize HLC: {}", e))?;
        drop(hlc_lock);

        let chunk_envelope = cosync_core::envelope::file_chunk_envelope(
            &device_id_bytes,
            &ts_bytes,
            &info.transfer_id,
            i as u32,
            chunk,
        );
        session
            .broadcast(&chunk_envelope)
            .await
            .map_err(|e| e.to_string())?;

        // Brief yield between chunks to let the QUIC connection breathe
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }

    // Send ack
    let hlc = session.hlc();
    let mut hlc_lock = hlc.lock().await;
    let ts = hlc_lock.now();
    let ts_bytes = serde_json::to_vec(&ts).map_err(|e| format!("Serialize HLC: {}", e))?;
    drop(hlc_lock);

    let ack_envelope = cosync_core::envelope::file_ack_envelope(
        &device_id_bytes,
        &ts_bytes,
        &info.transfer_id,
        true,
        "",
    );
    session
        .broadcast(&ack_envelope)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SendFileResult {
        transfer_id: info.transfer_id,
        file_name: info.file_name,
        file_size: info.file_size,
        total_chunks: info.total_chunks,
    })
}

/// Opens the parent folder of a file in the system file manager.
#[tauri::command]
pub async fn open_file_in_folder(file_path: String) -> Result<(), String> {
    let path = std::path::Path::new(&file_path);
    let parent = path
        .parent()
        .ok_or("Cannot determine parent directory")?;

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &file_path])
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }

    Ok(())
}

/// Returns this device's fingerprint.
#[tauri::command]
pub async fn get_device_fingerprint(state: State<'_, CosyncState>) -> Result<String, String> {
    let identity = state.identity.lock().await;
    identity.fingerprint().map_err(|e| e.to_string())
}