use cosync_core::{ConnectionState, DeviceIdentity, DiscoveryService, SessionManager};
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
    /// Path to the app data directory (for identity + database storage).
    pub app_data_dir: Arc<Mutex<PathBuf>>,
}

impl CosyncState {
    /// Creates state with a device identity loaded from (or created in) `data_dir`.
    pub fn new(data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&data_dir).expect("Failed to create app data dir");

        let identity = DeviceIdentity::load_or_create(&data_dir, &get_hostname())
            .expect("Failed to load or create device identity");

        Self {
            identity: Arc::new(Mutex::new(identity)),
            session_manager: Arc::new(Mutex::new(None)),
            discovery: Arc::new(Mutex::new(None)),
            connection_state: Arc::new(Mutex::new(ConnectionState::Idle)),
            app_data_dir: Arc::new(Mutex::new(data_dir)),
        }
    }
}

/// Falls back to "cosync-device" if hostname cannot be determined.
fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "cosync-device".to_string())
}

// We need the `hostname` crate for getting the machine name.
// It's a tiny crate (no dependencies) so we add it to Cargo.toml.