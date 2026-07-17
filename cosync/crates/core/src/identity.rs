use crate::error::{CosyncError, Result};
use ed25519_dalek::{SigningKey, pkcs8::EncodePrivateKey};
use std::str::FromStr;
use rand::rngs::OsRng;
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DeviceIdentity {
    pub signing_key: SigningKey,
    pub verifying_key: ed25519_dalek::VerifyingKey,
    pub cert_der: Vec<u8>,
    pub device_name: String,
    pub data_dir: PathBuf,
}

impl DeviceIdentity {
    pub fn load_or_create(data_dir: &std::path::Path, device_name: &str) -> Result<Self> {
        fs::create_dir_all(data_dir)?;
        let key_path = data_dir.join("identity_key.bin");
        let cert_path = data_dir.join("identity_cert.der");
        let meta_path = data_dir.join("identity_meta.json");
        if key_path.exists() && cert_path.exists() && meta_path.exists() {
            return Self::load(data_dir);
        }
        Self::create(data_dir, device_name)
    }

    fn create(data_dir: &std::path::Path, device_name: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let pkcs8_doc = signing_key.to_pkcs8_der()
            .map_err(|e| CosyncError::Cert(format!("PKCS#8 export failed: {}", e)))?;
        let key_pair = KeyPair::try_from(pkcs8_doc.as_bytes())
            .map_err(|e| CosyncError::Cert(format!("Failed to create KeyPair from PKCS#8: {}", e)))?;

        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, format!("Cosync_{}", device_name));
        params.distinguished_name = dn;
        params.subject_alt_names.push(rcgen::SanType::IpAddress(
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        ));
        params.subject_alt_names.push(rcgen::SanType::IpAddress(
            std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED),
        ));
        params.subject_alt_names.push(rcgen::SanType::DnsName(
            rcgen::Ia5String::from_str("localhost").unwrap(),
        ));

        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2034, 1, 1);

        let cert = params.self_signed(&key_pair)
            .map_err(|e| CosyncError::Cert(format!("Self-sign failed: {}", e)))?;
        let cert_der = cert.der().to_vec();

        fs::write(data_dir.join("identity_key.bin"), signing_key.to_bytes())?;
        fs::write(data_dir.join("identity_cert.der"), &cert_der)?;
        let meta = serde_json::json!({ "device_name": device_name, "key_type": "Ed25519" });
        fs::write(data_dir.join("identity_meta.json"), serde_json::to_string_pretty(&meta)?)?;

        Ok(Self {
            signing_key, verifying_key, cert_der,
            device_name: device_name.to_string(),
            data_dir: data_dir.to_path_buf(),
        })
    }

    fn load(data_dir: &std::path::Path) -> Result<Self> {
        let key_bytes = fs::read(data_dir.join("identity_key.bin"))?;
        let cert_der = fs::read(data_dir.join("identity_cert.der"))?;
        let meta_str = fs::read_to_string(data_dir.join("identity_meta.json"))?;
        let meta: serde_json::Value = serde_json::from_str(&meta_str)?;
        let signing_key = SigningKey::from_bytes(
            key_bytes.as_slice().try_into()
                .map_err(|_| CosyncError::Cert("Invalid Ed25519 key length".into()))?,
        );
        let verifying_key = signing_key.verifying_key();
        let device_name = meta["device_name"].as_str().unwrap_or("Unknown").to_string();
        Ok(Self { signing_key, verifying_key, cert_der, device_name, data_dir: data_dir.to_path_buf() })
    }

    pub fn compute_cert_fingerprint(cert_der: &[u8]) -> Result<String> {
        let parsed = x509_parser::parse_x509_certificate(cert_der)
            .map_err(|e| CosyncError::Cert(format!("Failed to parse cert: {}", e)))?;
        let mut hasher = Sha256::new();
        hasher.update(parsed.1.public_key().raw);
        Ok(hex::encode(hasher.finalize()))
    }

    pub fn fingerprint(&self) -> Result<String> {
        Self::compute_cert_fingerprint(&self.cert_der)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_load_identity() {
        let tmp = TempDir::new().unwrap();
        let id = DeviceIdentity::load_or_create(tmp.path(), "test-device").unwrap();
        assert_eq!(id.device_name, "test-device");
        let id2 = DeviceIdentity::load_or_create(tmp.path(), "test-device").unwrap();
        assert_eq!(id.cert_der, id2.cert_der);
        assert_eq!(id.fingerprint().unwrap(), id2.fingerprint().unwrap());
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let tmp = TempDir::new().unwrap();
        let id = DeviceIdentity::load_or_create(tmp.path(), "fp-test").unwrap();
        assert_eq!(id.fingerprint().unwrap().len(), 64);
    }

    #[test]
    fn test_different_devices_different_keys() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        let id1 = DeviceIdentity::load_or_create(tmp1.path(), "d1").unwrap();
        let id2 = DeviceIdentity::load_or_create(tmp2.path(), "d2").unwrap();
        assert_ne!(id1.fingerprint().unwrap(), id2.fingerprint().unwrap());
    }
}