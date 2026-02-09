use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct ChronosStore {
    db: Arc<Db>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub timestamp: i64,
    pub size: usize,
}

impl ChronosStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn save_snapshot(&self, file_path: &str, content: &[u8]) -> Result<()> {
        let compressed = zstd::encode_all(content, 0)?;
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 1. Primary Data Key: "file_path:timestamp" (Existing format)
        let data_key = format!("{}:{}", file_path, timestamp);

        // 2. Time Index Key: "__time_idx__:timestamp:file_path"
        let time_key = format!("__time_idx__:{}:{}", timestamp, file_path);

        // Use a batch to ensure atomicity
        let mut batch = sled::Batch::default();
        batch.insert(data_key.as_bytes(), compressed.as_slice());
        batch.insert(time_key.as_bytes(), &[]); // Empty value for index
        self.db.apply_batch(batch)?;

        Ok(())
    }

    pub fn get_history(&self, file_path: &str) -> Result<Vec<SnapshotInfo>> {
        let prefix = format!("{}:", file_path);
        let mut snapshots = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);

            // Extract timestamp from key after the last colon
            if let Some(ts_str) = key_str.rsplit(':').next() {
                if let Ok(ts) = ts_str.parse::<i64>() {
                    snapshots.push(SnapshotInfo {
                        timestamp: ts,
                        size: value.len(),
                    });
                }
            }
        }

        // Sort by timestamp descending (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(snapshots)
    }

    pub fn get_snapshot(&self, file_path: &str, timestamp: i64) -> Result<Option<Vec<u8>>> {
        let key = format!("{}:{}", file_path, timestamp);
        if let Some(compressed) = self.db.get(key.as_bytes())? {
            let content = zstd::decode_all(compressed.as_ref())?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Retrieve a timeline of all file changes across the repository.
    /// Returns a list of (timestamp, file_path) tuples, sorted newest first.
    pub fn get_global_timeline(&self, limit: usize) -> Result<Vec<(i64, String)>> {
        let prefix = "__time_idx__:";
        let mut events = Vec::new();

        // Scan backwards from the end if possible, or scan all and take last N
        // sled scan is lexicographical. Timestamp is i64, so string sort works if padded...
        // Wait, timestamp is standard i64 string. Variable length hurts sorting: 10 vs 2.
        // Assuming timestamps are current epoch millis (13 digits), string sort is fine.

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item?;
            let key_str = String::from_utf8_lossy(&key);

            // Format: __time_idx__:TIMESTAMP:PATH
            let parts: Vec<&str> = key_str.splitn(3, ':').collect();
            if parts.len() == 3 {
                if let Ok(ts) = parts[1].parse::<i64>() {
                    let path = parts[2].to_string();
                    events.push((ts, path));
                }
            }
        }

        // Sort descending (newest first)
        events.sort_by(|a, b| b.0.cmp(&a.0));
        events.truncate(limit);

        Ok(events)
    }

    /// Recursively find the state of all tracked files at `target_timestamp`
    /// and return a map of { file_path -> content }.
    /// This allows reconstructing a "Ghost Branch" state.
    pub fn get_checkpoint_state(&self, target_timestamp: i64) -> Result<Vec<(String, Vec<u8>)>> {
        // 1. Identify all unique files that existed <= target_timestamp.
        // We scan the global timeline.
        let prefix = "__time_idx__:";
        let mut unique_files = std::collections::HashSet::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item?;
            let key_str = String::from_utf8_lossy(&key);
            let parts: Vec<&str> = key_str.splitn(3, ':').collect();
            if parts.len() == 3 {
                if let Ok(ts) = parts[1].parse::<i64>() {
                    // Only consider files modified BEFORE or AT target
                    if ts <= target_timestamp {
                        unique_files.insert(parts[2].to_string());
                    }
                }
            }
        }

        let mut restored_files = Vec::new();

        // 2. For each unique file, find its latest snapshot <= target_timestamp
        for path in unique_files {
            // Scan this file's history to find best match
            // Optimization: We could use range query on "path:timestamp" if we knew the timestamp.
            // Since we don't, we just get history and find max(t) <= target.

            // Reuse get_history for simplicity inside this module
            // BUT get_history parses all keys. Let's do a more direct scan if possible.
            // Actually, get_history is fast enough for typical use cases per file.

            // Manual scan for efficiency: Scan "path:" and stop when ts > target?
            // Keys are "path:timestamp". Timestamp string sorting.
            // 100 < 20. Need to be careful.
            // Let's rely on get_history's logic for now as it's robust.

            let history = self.get_history(&path)?;
            // History is sorted desc. Find first (newest) that is <= target.
            if let Some(snapshot) = history.iter().find(|s| s.timestamp <= target_timestamp) {
                if let Ok(Some(content)) = self.get_snapshot(&path, snapshot.timestamp) {
                    restored_files.push((path, content));
                }
            }
        }

        Ok(restored_files)
    }
}

pub fn init_db() {
    // Placeholder
}
