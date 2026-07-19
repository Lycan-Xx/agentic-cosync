pub mod clipboard;
pub mod discovery;
pub mod envelope;
pub mod error;
pub mod file_transfer;
pub mod hlc;
pub mod identity;
pub mod notification;
pub mod pairing;
pub mod session;
pub mod state;
pub mod storage;
pub mod transport;
pub mod wire;

pub use discovery::DiscoveryService;
pub use error::CosyncError;
pub use hlc::{HlcTimestamp, HybridLogicalClock};
pub use identity::DeviceIdentity;
pub use session::{SessionEvent, SessionManager};
pub use state::ConnectionState;
pub use storage::Storage;

#[cfg(feature = "mobile-bindings")]
uniffi::setup_scaffolding!();