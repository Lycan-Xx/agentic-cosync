use thiserror::Error;

#[derive(Error, Debug)]
pub enum CosyncError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("Certificate error: {0}")]
    Cert(String),
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Discovery error: {0}")]
    Discovery(String),
    #[error("Pairing error: {0}")]
    Pairing(String),
    #[error("Clipboard error: {0}")]
    Clipboard(String),
    #[error("File transfer error: {0}")]
    FileTransfer(String),
    #[error("Not connected to any peer")]
    NotConnected,
    #[error("Loop detected: update originated from this device")]
    LoopDetected,
    #[error("Stale update: received timestamp is older than local")]
    StaleUpdate,
    #[error("Size limit exceeded: {0} bytes exceeds {1} byte limit")]
    SizeLimitExceeded(usize, usize),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CosyncError>;