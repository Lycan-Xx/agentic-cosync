use crate::error::Result;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairingPayload {
    pub device_name: String,
    pub public_key_fingerprint: String,
    pub ip_hint: String,
    pub port: u32,
    pub pairing_token: String,
}

impl PairingPayload {
    pub fn new(device_name: String, public_key_fingerprint: String, ip_hint: String, port: u32) -> Self {
        let mut token_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut token_bytes);
        Self {
            device_name, public_key_fingerprint, ip_hint, port,
            pairing_token: hex::encode(token_bytes),
        }
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let p = PairingPayload::new("My Laptop".into(),
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".into(),
            "192.168.1.42".into(), 9443);
        let json = p.to_json().unwrap();
        let decoded = PairingPayload::from_json(&json).unwrap();
        assert_eq!(p, decoded);
        assert_eq!(decoded.pairing_token.len(), 64);
    }

    #[test]
    fn test_invalid_json() {
        assert!(PairingPayload::from_json("not json").is_err());
    }
}