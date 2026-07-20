use crate::error::{CosyncError, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .map_err(|e| CosyncError::Storage(format!("Open db: {}", e)))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| CosyncError::Storage(format!("WAL: {}", e)))?;
        Self::create_tables(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn create_tables(conn: &Connection) -> Result<()> {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS paired_devices (
                device_id TEXT PRIMARY KEY, device_name TEXT NOT NULL,
                fingerprint TEXT NOT NULL, last_known_ip TEXT,
                last_seen_at TEXT NOT NULL DEFAULT (datetime('now')));
            CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT, content BLOB NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'text/plain',
                source_device_id TEXT, hlc_time TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (source_device_id) REFERENCES paired_devices(device_id));
            CREATE INDEX IF NOT EXISTS idx_clipboard_created_at ON clipboard_history(created_at DESC);
        ").map_err(|e| CosyncError::Storage(format!("Create tables: {}", e)))?;
        Ok(())
    }

    pub fn upsert_device(&self, device_id: &str, device_name: &str, fingerprint: &str, last_known_ip: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        conn.execute("INSERT INTO paired_devices (device_id, device_name, fingerprint, last_known_ip, last_seen_at)
            VALUES (?1,?2,?3,?4,datetime('now'))
            ON CONFLICT(device_id) DO UPDATE SET device_name=?2, fingerprint=?3,
            last_known_ip=COALESCE(?4,last_known_ip), last_seen_at=datetime('now')",
            params![device_id, device_name, fingerprint, last_known_ip])
            .map_err(|e| CosyncError::Storage(format!("Upsert device: {}", e)))?;
        Ok(())
    }

    pub fn remove_device(&self, device_id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        conn.execute("DELETE FROM paired_devices WHERE device_id=?1", params![device_id])
            .map_err(|e| CosyncError::Storage(format!("Remove device: {}", e)))?;
        Ok(())
    }

    pub fn get_devices(&self) -> Result<Vec<PairedDevice>> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT device_id,device_name,fingerprint,last_known_ip,last_seen_at FROM paired_devices ORDER BY last_seen_at DESC")
            .map_err(|e| CosyncError::Storage(format!("Query devices: {}", e)))?;
        let rows = stmt.query_map([], |r| Ok(PairedDevice { device_id: r.get(0)?, device_name: r.get(1)?,
            fingerprint: r.get(2)?, last_known_ip: r.get(3)?, last_seen_at: r.get(4)? }))
            .map_err(|e| CosyncError::Storage(e.to_string()))?;
        let mut devs = Vec::new();
        for r in rows { devs.push(r.map_err(|e| CosyncError::Storage(e.to_string()))?); }
        Ok(devs)
    }

    pub fn get_device(&self, device_id: &str) -> Result<Option<PairedDevice>> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT device_id,device_name,fingerprint,last_known_ip,last_seen_at FROM paired_devices WHERE device_id=?1")
            .map_err(|e| CosyncError::Storage(format!("Query device: {}", e)))?;
        let mut rows = stmt.query_map(params![device_id], |r| Ok(PairedDevice { device_id: r.get(0)?, device_name: r.get(1)?,
            fingerprint: r.get(2)?, last_known_ip: r.get(3)?, last_seen_at: r.get(4)? }))
            .map_err(|e| CosyncError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(d)) => Ok(Some(d)),
            Some(Err(e)) => Err(CosyncError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    pub fn insert_clipboard(&self, content: &[u8], content_type: &str, source_device_id: Option<&str>, hlc_time: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        conn.execute("INSERT INTO clipboard_history (content,content_type,source_device_id,hlc_time,created_at) VALUES (?1,?2,?3,?4,datetime('now'))",
            params![content, content_type, source_device_id, hlc_time])
            .map_err(|e| CosyncError::Storage(format!("Insert clipboard: {}", e)))?;
        let rowid = conn.last_insert_rowid();
        conn.execute("DELETE FROM clipboard_history WHERE id NOT IN (SELECT id FROM clipboard_history ORDER BY created_at DESC LIMIT 100)", [])
            .map_err(|e| CosyncError::Storage(format!("Evict: {}", e)))?;
        Ok(rowid)
    }

    pub fn get_clipboard_history(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT id,content,content_type,source_device_id,hlc_time,created_at FROM clipboard_history ORDER BY created_at DESC LIMIT ?1")
            .map_err(|e| CosyncError::Storage(format!("Query history: {}", e)))?;
        let rows = stmt.query_map(params![limit as i64], |r| Ok(ClipboardEntry { id: r.get(0)?, content: r.get(1)?,
            content_type: r.get(2)?, source_device_id: r.get(3)?, hlc_time: r.get(4)?, created_at: r.get(5)? }))
            .map_err(|e| CosyncError::Storage(e.to_string()))?;
        let mut entries = Vec::new();
        for r in rows { entries.push(r.map_err(|e| CosyncError::Storage(e.to_string()))?); }
        Ok(entries)
    }

    pub fn search_clipboard(&self, query: &str, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare("SELECT id,content,content_type,source_device_id,hlc_time,created_at FROM clipboard_history WHERE CAST(content AS TEXT) LIKE ?1 ORDER BY created_at DESC LIMIT ?2")
            .map_err(|e| CosyncError::Storage(format!("Search: {}", e)))?;
        let rows = stmt.query_map(params![pattern, limit as i64], |r| Ok(ClipboardEntry { id: r.get(0)?, content: r.get(1)?,
            content_type: r.get(2)?, source_device_id: r.get(3)?, hlc_time: r.get(4)?, created_at: r.get(5)? }))
            .map_err(|e| CosyncError::Storage(e.to_string()))?;
        let mut entries = Vec::new();
        for r in rows { entries.push(r.map_err(|e| CosyncError::Storage(e.to_string()))?); }
        Ok(entries)
    }

    pub fn clear_clipboard_history(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        conn.execute("DELETE FROM clipboard_history", []).map_err(|e| CosyncError::Storage(format!("Clear: {}", e)))?;
        Ok(())
    }

    pub fn delete_clipboard_entry(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| CosyncError::Storage(e.to_string()))?;
        conn.execute("DELETE FROM clipboard_history WHERE id=?1", params![id])
            .map_err(|e| CosyncError::Storage(format!("Delete entry: {}", e)))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PairedDevice { pub device_id: String, pub device_name: String, pub fingerprint: String, pub last_known_ip: Option<String>, pub last_seen_at: String }

#[derive(Debug, Clone)]
pub struct ClipboardEntry { pub id: i64, pub content: Vec<u8>, pub content_type: String, pub source_device_id: Option<String>, pub hlc_time: Option<String>, pub created_at: String }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Storage) {
        let tmp = TempDir::new().unwrap();
        let s = Storage::open(&tmp.path().join("test.db")).unwrap();
        (tmp, s)
    }

    #[test]
    fn test_create_tables() {
        let (_, s) = setup();
        assert!(s.get_devices().unwrap().is_empty());
    }

    #[test]
    fn test_upsert_and_get_device() {
        let (_, s) = setup();
        s.upsert_device("d1", "Laptop", "fp-abc", Some("192.168.1.10")).unwrap();
        let d = s.get_devices().unwrap();
        assert_eq!(d.len(), 1); assert_eq!(d[0].device_name, "Laptop");
        s.upsert_device("d1", "Laptop Pro", "fp-abc", Some("192.168.1.11")).unwrap();
        let d = s.get_devices().unwrap();
        assert_eq!(d[0].device_name, "Laptop Pro");
    }

    #[test]
    fn test_remove_device() {
        let (_, s) = setup();
        s.upsert_device("d1", "Laptop", "fp", None).unwrap();
        s.remove_device("d1").unwrap();
        assert!(s.get_devices().unwrap().is_empty());
    }

    #[test]
    fn test_clipboard_history() {
        let (_, s) = setup();
        s.upsert_device("d1", "L", "fp", None).unwrap();
        s.insert_clipboard(b"hello world", "text/plain", Some("d1"), None).unwrap();
        s.insert_clipboard(b"hello rust", "text/plain", None, None).unwrap();
        let h = s.get_clipboard_history(10).unwrap();
        assert_eq!(h.len(), 2);
        let has_hw = h.iter().any(|e| e.content == b"hello world");
        let has_hr = h.iter().any(|e| e.content == b"hello rust");
        assert!(has_hw); assert!(has_hr);
    }

    #[test]
    fn test_clipboard_eviction() {
        let (_, s) = setup();
        for i in 0..110 { s.insert_clipboard(format!("e-{}", i).as_bytes(), "text/plain", None, None).unwrap(); }
        assert_eq!(s.get_clipboard_history(200).unwrap().len(), 100);
    }

    #[test]
    fn test_clipboard_search() {
        let (_, s) = setup();
        s.insert_clipboard(b"hello world", "text/plain", None, None).unwrap();
        s.insert_clipboard(b"goodbye world", "text/plain", None, None).unwrap();
        assert_eq!(s.search_clipboard("hello", 10).unwrap().len(), 1);
    }
}