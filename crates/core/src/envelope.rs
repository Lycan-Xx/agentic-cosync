use crate::error::{CosyncError, Result};
use crate::wire::cosync::*;
use prost::Message;

pub fn encode_envelope(envelope: &Envelope) -> Result<Vec<u8>> {
    Ok(envelope.encode_to_vec())
}

pub fn decode_envelope(data: &[u8]) -> Result<Envelope> {
    Envelope::decode(data).map_err(|e| CosyncError::Decode(format!("Failed to decode: {}", e)))
}

pub fn heartbeat_envelope(sender_id: &[u8], hlc_bytes: &[u8], seq: u64) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::Heartbeat(Heartbeat { seq })) }
}

pub fn clipboard_envelope(sender_id: &[u8], hlc_bytes: &[u8], content: &[u8], content_type: &str) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::ClipboardUpdate(ClipboardUpdate {
            content: content.to_vec(), content_type: content_type.to_string() })) }
}

pub fn file_meta_envelope(sender_id: &[u8], hlc_bytes: &[u8], file_name: &str, file_size: u64,
    sha256: &str, content_type: &str, total_chunks: u32, transfer_id: &str) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::FileMeta(FileMeta {
            file_name: file_name.into(), file_size, sha256: sha256.into(),
            content_type: content_type.into(), total_chunks, transfer_id: transfer_id.into() })) }
}

pub fn file_chunk_envelope(sender_id: &[u8], hlc_bytes: &[u8], transfer_id: &str, chunk_index: u32, data: &[u8]) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::FileChunk(FileChunk {
            transfer_id: transfer_id.into(), chunk_index, data: data.to_vec() })) }
}

pub fn file_ack_envelope(sender_id: &[u8], hlc_bytes: &[u8], transfer_id: &str, success: bool, error: &str) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::FileAck(FileAck {
            transfer_id: transfer_id.into(), success, error: error.into() })) }
}

pub fn notification_event_envelope(sender_id: &[u8], hlc_bytes: &[u8], package_name: &str,
    title: &str, text: &str, extras: std::collections::HashMap<String, String>, posted_at_ms: i64) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::NotificationEvent(NotificationEvent {
            package_name: package_name.into(), title: title.into(), text: text.into(), extras, posted_at_ms })) }
}

pub fn notification_reply_envelope(sender_id: &[u8], hlc_bytes: &[u8], notification_key: &str, reply_text: &str) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::NotificationReply(NotificationReply {
            notification_key: notification_key.into(), reply_text: reply_text.into() })) }
}

pub fn pairing_request_envelope(sender_id: &[u8], hlc_bytes: &[u8], device_name: &str,
    fingerprint: &str, port: u32, token: &[u8]) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::PairingRequest(PairingRequest {
            device_name: device_name.into(), public_key_fingerprint: fingerprint.into(), port, pairing_token: token.to_vec() })) }
}

pub fn pairing_ack_envelope(sender_id: &[u8], hlc_bytes: &[u8], accepted: bool, reason: &str,
    device_name: &str, fingerprint: &str) -> Envelope {
    Envelope { sender_device_id: sender_id.to_vec(), hlc_timestamp: hlc_bytes.to_vec(),
        payload: Some(envelope::Payload::PairingAck(PairingAck {
            accepted, reason: reason.into(), device_name: device_name.into(), public_key_fingerprint: fingerprint.into() })) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_heartbeat() {
        let env = heartbeat_envelope(b"dev-1", b"{}", 42);
        let bytes = encode_envelope(&env).unwrap();
        let decoded = decode_envelope(&bytes).unwrap();
        assert_eq!(env.sender_device_id, decoded.sender_device_id);
        match decoded.payload {
            Some(envelope::Payload::Heartbeat(hb)) => assert_eq!(hb.seq, 42),
            _ => panic!("Expected Heartbeat"),
        }
    }

    #[test]
    fn test_encode_decode_clipboard() {
        let env = clipboard_envelope(b"dev-1", b"{}", b"hello", "text/plain");
        let bytes = encode_envelope(&env).unwrap();
        let decoded = decode_envelope(&bytes).unwrap();
        match decoded.payload {
            Some(envelope::Payload::ClipboardUpdate(cb)) => {
                assert_eq!(cb.content, b"hello");
                assert_eq!(cb.content_type, "text/plain");
            }
            _ => panic!("Expected ClipboardUpdate"),
        }
    }

    #[test]
    fn test_encode_decode_file_meta() {
        let env = file_meta_envelope(b"dev-1", b"{}", "photo.jpg", 1024, "abc", "image/jpeg", 16, "tx-001");
        let bytes = encode_envelope(&env).unwrap();
        let decoded = decode_envelope(&bytes).unwrap();
        match decoded.payload {
            Some(envelope::Payload::FileMeta(fm)) => {
                assert_eq!(fm.file_name, "photo.jpg");
                assert_eq!(fm.total_chunks, 16);
            }
            _ => panic!("Expected FileMeta"),
        }
    }
}