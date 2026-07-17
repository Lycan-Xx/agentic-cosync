use crate::error::{CosyncError, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

const CHUNK_SIZE: usize = 64 * 1024;

pub struct FileTransferManager {
    incoming: tokio::sync::Mutex<HashMap<String, IncomingTransfer>>,
}

struct IncomingTransfer {
    temp_path: PathBuf,
    final_path: PathBuf,
    expected_sha256: String,
    received_chunks: u32,
}

#[derive(Debug, Clone)]
pub struct FileSendInfo {
    pub file_name: String,
    pub file_size: u64,
    pub sha256: String,
    pub total_chunks: u32,
    pub transfer_id: String,
}

impl FileTransferManager {
    pub fn new() -> Self { Self { incoming: tokio::sync::Mutex::new(HashMap::new()) } }

    pub async fn send_file(&self, file_path: &std::path::Path, transfer_id: &str) -> Result<(FileSendInfo, Vec<Vec<u8>>)> {
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len() as u64;
        let file_name = file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("unknown".into());
        let mut file = File::open(file_path).await?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; CHUNK_SIZE];
        loop { let n = file.read(&mut buf).await?; if n == 0 { break; } hasher.update(&buf[..n]); }
        let sha256 = hex::encode(hasher.finalize());
        let total_chunks = (file_size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;
        file = File::open(file_path).await?;
        let mut chunks = Vec::with_capacity(total_chunks);
        loop {
            let mut chunk_buf = vec![0u8; CHUNK_SIZE];
            let n = file.read(&mut chunk_buf).await?;
            if n == 0 { break; }
            chunk_buf.truncate(n);
            chunks.push(chunk_buf);
        }
        Ok((FileSendInfo { file_name, file_size, sha256, total_chunks: total_chunks as u32, transfer_id: transfer_id.into() }, chunks))
    }

    pub async fn begin_receive(&self, transfer_id: &str, file_name: &str, _file_size: u64,
        expected_sha256: &str, _total_chunks: u32, download_dir: &std::path::Path) -> Result<()> {
        let temp_path = download_dir.join(format!(".cosync-tmp-{}", transfer_id));
        let final_path = download_dir.join(file_name);
        File::create(&temp_path).await?.set_len(0).await?;
        self.incoming.lock().await.insert(transfer_id.to_string(), IncomingTransfer {
            temp_path, final_path, expected_sha256: expected_sha256.to_string(), received_chunks: 0 });
        Ok(())
    }

    pub async fn receive_chunk(&self, transfer_id: &str, chunk_index: u32, data: &[u8]) -> Result<u32> {
        let mut incoming = self.incoming.lock().await;
        let transfer = incoming.get_mut(transfer_id)
            .ok_or_else(|| CosyncError::FileTransfer(format!("Unknown transfer: {}", transfer_id)))?;
        let offset = chunk_index as u64 * CHUNK_SIZE as u64;
        let mut file = File::options().write(true).open(&transfer.temp_path).await?;
        file.seek(SeekFrom::Start(offset)).await?;
        file.write_all(data).await?;
        file.flush().await?;
        transfer.received_chunks += 1;
        Ok(transfer.received_chunks)
    }

    pub async fn complete_receive(&self, transfer_id: &str) -> Result<std::path::PathBuf> {
        let mut incoming = self.incoming.lock().await;
        let transfer = incoming.remove(transfer_id)
            .ok_or_else(|| CosyncError::FileTransfer(format!("Unknown transfer: {}", transfer_id)))?;
        let mut file = File::open(&transfer.temp_path).await?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; CHUNK_SIZE];
        loop { let n = file.read(&mut buf).await?; if n == 0 { break; } hasher.update(&buf[..n]); }
        let actual = hex::encode(hasher.finalize());
        if actual != transfer.expected_sha256 {
            let _ = tokio::fs::remove_file(&transfer.temp_path).await;
            return Err(CosyncError::FileTransfer(format!("SHA-256 mismatch: expected {}, got {}", transfer.expected_sha256, actual)));
        }
        tokio::fs::rename(&transfer.temp_path, &transfer.final_path).await?;
        Ok(transfer.final_path)
    }

    pub async fn cancel_receive(&self, transfer_id: &str) -> Result<()> {
        let mut incoming = self.incoming.lock().await;
        if let Some(t) = incoming.remove(transfer_id) { let _ = tokio::fs::remove_file(&t.temp_path).await; }
        Ok(())
    }
}

impl Default for FileTransferManager { fn default() -> Self { Self::new() } }