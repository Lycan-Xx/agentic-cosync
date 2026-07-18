use cosync_core::{ConnectionState, DeviceIdentity, DiscoveryService, SessionEvent, SessionManager, Storage};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state, managed by Tauri's state system.
/// Every Tauri command receives a `State<'_, CosyncState>` reference.
pub struct CosyncState {
    /// This device's cryptographic identity (keypair + self-signed cert).
    pub identity: Arc<Mutex<DeviceIdentity>>,
    /// The session manager handles QUIC connections, pairing, and data sync.
    pub session_manager: Arc<Mutex<Option<SessionManager>>>,
    /// The mDNS discovery service for finding peers on the LAN.
    pub discovery: Arc<Mutex<Option<DiscoveryService>>>,
    /// The current connection state, readable by the frontend.
    pub connection_state: Arc<Mutex<ConnectionState>>,
    /// SQLite storage — always available, even before a session is started.
    pub storage: Arc<Storage>,
    /// Path to the app data directory (for identity + database storage + received files).
    pub app_data_dir: Arc<Mutex<PathBuf>>,
}

impl CosyncState {
    /// Creates state with a device identity loaded from (or created in) `data_dir`.
    pub fn new(data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&data_dir).expect("Failed to create app data dir");

        let identity = DeviceIdentity::load_or_create(&data_dir, &get_hostname())
            .expect("Failed to load or create device identity");

        let storage =
            Storage::open(&data_dir.join("cosync.db")).expect("Failed to open storage");

        Self {
            identity: Arc::new(Mutex::new(identity)),
            session_manager: Arc::new(Mutex::new(None)),
            discovery: Arc::new(Mutex::new(None)),
            connection_state: Arc::new(Mutex::new(ConnectionState::Idle)),
            storage: Arc::new(storage),
            app_data_dir: Arc::new(Mutex::new(data_dir)),
        }
    }
}

/// Converts a core `SessionEvent` into a `FrontendEvent` for the JS frontend.
pub async fn session_event_to_frontend(fe: SessionEvent) -> crate::commands::FrontendEvent {
    match fe {
        SessionEvent::StateChanged(cs) => crate::commands::FrontendEvent::ConnectionStateChanged {
            state: format!("{:?}", cs),
        },
        SessionEvent::ClipboardReceived { content, content_type, source } => {
            let text = if content_type.starts_with("text/") {
                String::from_utf8_lossy(&content).to_string()
            } else {
                format!("[{} attachment, {} bytes]", content_type, content.len())
            };
            crate::commands::FrontendEvent::ClipboardReceived {
                content: text,
                source,
            }
        }
        SessionEvent::FileIncoming { transfer_id, file_name, file_size } => {
            crate::commands::FrontendEvent::FileIncoming {
                transfer_id,
                file_name,
                file_size,
            }
        }
        SessionEvent::FileProgress { transfer_id, chunk_index, total_chunks } => {
            crate::commands::FrontendEvent::FileProgress {
                transfer_id,
                chunk_index,
                total_chunks,
            }
        }
        SessionEvent::FileComplete { transfer_id, success, path } => {
            crate::commands::FrontendEvent::FileComplete {
                transfer_id,
                success,
                path,
            }
        }
        SessionEvent::NotificationReceived { package_name, title, text } => {
            crate::commands::FrontendEvent::NotificationReceived {
                package_name,
                title,
                text,
            }
        }
        SessionEvent::PeerPaired { device_name, fingerprint } => {
            crate::commands::FrontendEvent::PairingRequest {
                device_name,
                fingerprint,
            }
        }
        SessionEvent::Error(msg) => crate::commands::FrontendEvent::Error { message: msg },
    }
}

/// Falls back to "cosync-device" if hostname cannot be determined.
fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "cosync-device".to_string())
}