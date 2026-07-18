use crate::envelope::clipboard_envelope;
use crate::error::{CosyncError, Result};
use crate::hlc::HybridLogicalClock;
use crate::session::SessionManager;
use crate::storage::Storage;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

const MAX_CLIPBOARD_SIZE: usize = 5 * 1024 * 1024;
const DEDUP_BUFFER_SIZE: usize = 10;

pub struct ClipboardSync {
    max_size: usize,
    recent_hashes: VecDeque<String>,
    hlc: Arc<Mutex<HybridLogicalClock>>,
    device_id: Vec<u8>,
    storage: Arc<Storage>,
}

impl ClipboardSync {
    pub fn new(hlc: Arc<Mutex<HybridLogicalClock>>, device_id: Vec<u8>, storage: Arc<Storage>) -> Self {
        Self { max_size: MAX_CLIPBOARD_SIZE, recent_hashes: VecDeque::with_capacity(DEDUP_BUFFER_SIZE),
            hlc, device_id, storage }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self { self.max_size = max_size; self }

    pub async fn send_clipboard(&mut self, content: &[u8], content_type: &str, session: &SessionManager) -> Result<()> {
        if content.len() > self.max_size {
            return Err(CosyncError::SizeLimitExceeded(content.len(), self.max_size));
        }
        let hash = Self::content_hash(content);
        if self.recent_hashes.contains(&hash) { return Ok(()); }
        self.recent_hashes.push_back(hash);
        if self.recent_hashes.len() > DEDUP_BUFFER_SIZE { self.recent_hashes.pop_front(); }
        self.storage.insert_clipboard(content, content_type, None, None)?;
        let mut hlc = self.hlc.lock().await;
        let ts = hlc.now();
        let ts_bytes = serde_json::to_vec(&ts)?;
        drop(hlc);
        let envelope = clipboard_envelope(&self.device_id, &ts_bytes, content, content_type);
        session.broadcast(&envelope).await
    }

    pub async fn apply_clipboard(&mut self, content: &[u8], content_type: &str, hlc_ts: &crate::hlc::HlcTimestamp) -> Result<()> {
        if content.len() > self.max_size {
            return Err(CosyncError::SizeLimitExceeded(content.len(), self.max_size));
        }
        let hash = Self::content_hash(content);
        if self.recent_hashes.contains(&hash) { return Ok(()); }
        let mut hlc = self.hlc.lock().await;
        if !hlc.is_newer_than(hlc_ts) { return Ok(()); }
        let _ = hlc.receive(hlc_ts);
        drop(hlc);
        #[cfg(not(feature = "mobile-bindings"))]
        {
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|e| CosyncError::Clipboard(format!("Open clipboard: {}", e)))?;
            if content_type.starts_with("text/") {
                let text = String::from_utf8_lossy(content).to_string();
                clipboard.set_text(text).map_err(|e| CosyncError::Clipboard(format!("Set text: {}", e)))?;
            } else if content_type == "image/png" {
                clipboard.set_image(arboard::ImageData { width: 1, height: 1,
                    bytes: std::borrow::Cow::Owned(content.to_vec()) })
                    .map_err(|e| CosyncError::Clipboard(format!("Set image: {}", e)))?;
            }
        }
        self.recent_hashes.push_back(hash);
        if self.recent_hashes.len() > DEDUP_BUFFER_SIZE { self.recent_hashes.pop_front(); }
        Ok(())
    }

    fn content_hash(content: &[u8]) -> String {
        let mut h = Sha256::new(); h.update(content); hex::encode(h.finalize())
    }
}

#[cfg(not(feature = "mobile-bindings"))]
pub struct DesktopClipboardMonitor { last_hash: String }

#[cfg(not(feature = "mobile-bindings"))]
impl DesktopClipboardMonitor {
    pub fn new() -> Self { Self { last_hash: String::new() } }

    pub fn poll(&mut self) -> Result<Option<(Vec<u8>, String)>> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| CosyncError::Clipboard(format!("Open: {}", e)))?;
        match clipboard.get_text() {
            Ok(text) => {
                let hash = ClipboardSync::content_hash(text.as_bytes());
                if hash == self.last_hash { return Ok(None); }
                self.last_hash = hash;
                Ok(Some((text.into_bytes(), "text/plain".to_string())))
            }
            Err(_) => {
                match clipboard.get_image() {
                    Ok(image) => {
                        let bytes: Vec<u8> = image.bytes.into();
                        let hash = ClipboardSync::content_hash(&bytes);
                        if hash == self.last_hash { return Ok(None); }
                        self.last_hash = hash;
                        Ok(Some((bytes, "image/png".to_string())))
                    }
                    Err(_) => Ok(None),
                }
            }
        }
    }

    pub fn reset(&mut self) { self.last_hash.clear(); }
}

#[cfg(not(feature = "mobile-bindings"))]
impl Default for DesktopClipboardMonitor {
    fn default() -> Self { Self::new() }
}