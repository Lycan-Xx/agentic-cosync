use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    Idle,
    Discovering,
    PeerFound(String),
    Pairing,
    Connected(String),
    Reconnecting(String),
    Error(String),
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Idle => write!(f, "Idle"),
            ConnectionState::Discovering => write!(f, "Discovering"),
            ConnectionState::PeerFound(n) => write!(f, "Peer found: {}", n),
            ConnectionState::Pairing => write!(f, "Pairing"),
            ConnectionState::Connected(n) => write!(f, "Connected to {}", n),
            ConnectionState::Reconnecting(n) => write!(f, "Reconnecting to {}", n),
            ConnectionState::Error(m) => write!(f, "Error: {}", m),
        }
    }
}