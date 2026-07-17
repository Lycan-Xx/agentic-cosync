use crate::error::{CosyncError, Result};
use crate::envelope::{decode_envelope, encode_envelope};
use crate::wire::cosync::Envelope;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::SignatureScheme;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;

const MAX_IDLE_TIMEOUT_MS: u32 = 30_000;
const KEEP_ALIVE_MS: u32 = 10_000;
const MAX_BIDI_STREAMS: u32 = 100;
const MAX_ENVELOPE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug)]
pub struct PinnedCertVerifier {
    trusted_fingerprints: Arc<std::sync::RwLock<HashSet<String>>>,
}

impl PinnedCertVerifier {
    pub fn new() -> Self {
        Self { trusted_fingerprints: Arc::new(std::sync::RwLock::new(HashSet::new())) }
    }

    pub fn add_fingerprint(&self, fp: &str) {
        self.trusted_fingerprints.write().unwrap().insert(fp.to_lowercase());
    }

    pub fn remove_fingerprint(&self, fp: &str) {
        self.trusted_fingerprints.write().unwrap().remove(&fp.to_lowercase());
    }

    fn verify_pinned_cert(&self, end_cert: &CertificateDer<'_>) -> std::result::Result<(), rustls::Error> {
        let fps = self.trusted_fingerprints.read().unwrap();
        if fps.is_empty() {
            return Err(rustls::Error::General("No trusted fingerprints".into()));
        }
        let mut hasher = Sha256::new();
        hasher.update(end_cert.as_ref());
        let cert_hash = hex::encode(hasher.finalize());
        if fps.contains(&cert_hash) { Ok(()) }
        else { Err(rustls::Error::General(format!("Cert fingerprint {} not trusted", cert_hash))) }
    }
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(&self, end_entity: &CertificateDer<'_>, _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>, _ocsp_response: &[u8],
        _now: rustls_pki_types::UnixTime) -> std::result::Result<ServerCertVerified, rustls::Error> {
        self.verify_pinned_cert(end_entity)?;
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(&self, _msg: &[u8], _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(&self, _msg: &[u8], _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519, SignatureScheme::ECDSA_NISTP256_SHA256,
             SignatureScheme::ECDSA_NISTP384_SHA384, SignatureScheme::RSA_PSS_SHA256,
             SignatureScheme::RSA_PKCS1_SHA256]
    }
}

impl Default for PinnedCertVerifier {
    fn default() -> Self { Self::new() }
}

pub fn create_server_endpoint(bind_addr: SocketAddr, cert_der: Vec<u8>, key_der: Vec<u8>) -> Result<Endpoint> {
    let cert_chain = vec![CertificateDer::from(cert_der)];
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));
    let server_crypto = rustls::ServerConfig::builder().with_no_client_auth()
        .with_single_cert(cert_chain, key).map_err(|e| CosyncError::Tls(format!("Server TLS: {}", e)))?;
    let quic_crypto = quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)
        .map_err(|e| CosyncError::Tls(format!("QuicServerConfig: {}", e)))?;
    let mut transport = quinn::TransportConfig::default();
    transport.max_idle_timeout(Some(quinn::IdleTimeout::try_from(
        std::time::Duration::from_millis(MAX_IDLE_TIMEOUT_MS as u64)).unwrap()));
    transport.keep_alive_interval(Some(std::time::Duration::from_millis(KEEP_ALIVE_MS as u64)));
    transport.max_concurrent_bidi_streams(quinn::VarInt::from_u32(MAX_BIDI_STREAMS));
    let mut server_config = ServerConfig::with_crypto(Arc::new(quic_crypto));
    server_config.transport_config(Arc::new(transport));
    let endpoint = Endpoint::server(server_config, bind_addr)
        .map_err(|e| CosyncError::Transport(format!("Bind failed: {}", e)))?;
    tracing::info!(addr = %bind_addr, "QUIC server endpoint created");
    Ok(endpoint)
}

pub fn create_client_endpoint(cert_verifier: Arc<PinnedCertVerifier>) -> Result<Endpoint> {
    let crypto = rustls::ClientConfig::builder().dangerous()
        .with_custom_certificate_verifier(cert_verifier).with_no_client_auth();
    let quic_crypto = quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
        .map_err(|e| CosyncError::Tls(format!("QuicClientConfig: {}", e)))?;
    let mut client_config = quinn::ClientConfig::new(Arc::new(quic_crypto));
    let mut transport = quinn::TransportConfig::default();
    transport.max_idle_timeout(Some(quinn::IdleTimeout::try_from(
        std::time::Duration::from_millis(MAX_IDLE_TIMEOUT_MS as u64)).unwrap()));
    transport.keep_alive_interval(Some(std::time::Duration::from_millis(KEEP_ALIVE_MS as u64)));
    client_config.transport_config(Arc::new(transport));
    let endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
        .map_err(|e| CosyncError::Transport(format!("Client endpoint failed: {}", e)))?;
    Ok(endpoint)
}

pub async fn send_envelope(send: &mut quinn::SendStream, envelope: &Envelope) -> Result<()> {
    let data = encode_envelope(envelope)?;
    if data.len() > MAX_ENVELOPE_SIZE {
        return Err(CosyncError::SizeLimitExceeded(data.len(), MAX_ENVELOPE_SIZE));
    }
    send.write_all(&(data.len() as u32).to_be_bytes()).await
        .map_err(|e| CosyncError::Transport(e.to_string()))?;
    send.write_all(&data).await.map_err(|e| CosyncError::Transport(e.to_string()))?;
    send.finish().map_err(|e| CosyncError::Transport(e.to_string()))?;
    Ok(())
}

pub async fn recv_envelope(recv: &mut quinn::RecvStream) -> Result<Envelope> {
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await.map_err(|e| CosyncError::Transport(e.to_string()))?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_ENVELOPE_SIZE {
        return Err(CosyncError::SizeLimitExceeded(len, MAX_ENVELOPE_SIZE));
    }
    let mut data = vec![0u8; len];
    recv.read_exact(&mut data).await.map_err(|e| CosyncError::Transport(e.to_string()))?;
    decode_envelope(&data)
}

pub async fn accept_connection(endpoint: &Endpoint) -> Result<quinn::Connection> {
    while let Some(incoming) = endpoint.accept().await {
        match incoming.await {
            Ok(conn) => return Ok(conn),
            Err(e) => { tracing::warn!("Accept failed: {}", e); continue; }
        }
    }
    Err(CosyncError::Transport("No incoming connections".into()))
}