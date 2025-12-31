//! Storage API - Extension storage (local, sync, session)
//!
//! Provides persistent storage for extensions with quota management.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use parking_lot::RwLock;

/// Storage area type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageArea {
    /// Local storage (persisted to disk)
    Local,
    /// Sync storage (would sync across devices)
    Sync,
    /// Session storage (cleared when browser closes)
    Session,
    /// Managed storage (enterprise policies)
    Managed,
}

/// Storage quota in bytes
pub const QUOTA_BYTES_LOCAL: usize = 10 * 1024 * 1024; // 10 MB
pub const QUOTA_BYTES_SYNC: usize = 100 * 1024; // 100 KB
pub const QUOTA_BYTES_SESSION: usize = 10 * 1024 * 1024; // 10 MB
pub const MAX_ITEMS_SYNC: usize = 512;
pub const QUOTA_BYTES_PER_ITEM_SYNC: usize = 8 * 1024; // 8 KB

/// Storage change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChange {
    /// Old value (None if new)
    pub old_value: Option<Value>,
    /// New value (None if removed)
    pub new_value: Option<Value>,
}

/// Extension storage implementation
pub struct ExtensionStorage {
    /// Storage directory
    storage_dir: PathBuf,
    /// In-memory cache for each extension's storage
    cache: RwLock<HashMap<String, HashMap<StorageArea, HashMap<String, Value>>>>,
}

impl ExtensionStorage {
    /// Create a new extension storage
    pub fn new(storage_dir: PathBuf) -> Self {
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir).ok();
        }

        Self {
            storage_dir,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get storage file path for an extension
    fn storage_path(&self, extension_id: &str, area: StorageArea) -> PathBuf {
        let area_name = match area {
            StorageArea::Local => "local",
            StorageArea::Sync => "sync",
            StorageArea::Session => "session",
            StorageArea::Managed => "managed",
        };
        self.storage_dir
            .join(extension_id)
            .join(format!("{}.json", area_name))
    }

    /// Load storage from disk
    fn load_storage(&self, extension_id: &str, area: StorageArea) -> HashMap<String, Value> {
        let path = self.storage_path(extension_id, area);
        
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        }
    }

    /// Save storage to disk
    fn save_storage(
        &self,
        extension_id: &str,
        area: StorageArea,
        data: &HashMap<String, Value>,
    ) -> Result<(), StorageError> {
        // Session storage is never persisted
        if area == StorageArea::Session {
            return Ok(());
        }

        let path = self.storage_path(extension_id, area);
        
        // Create directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(data)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;

        fs::write(&path, content)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Ensure extension storage is loaded
    fn ensure_loaded(&self, extension_id: &str, area: StorageArea) {
        let mut cache = self.cache.write();
        
        if !cache.contains_key(extension_id) {
            cache.insert(extension_id.to_string(), HashMap::new());
        }

        let ext_cache = cache.get_mut(extension_id).unwrap();
        if !ext_cache.contains_key(&area) {
            let data = self.load_storage(extension_id, area);
            ext_cache.insert(area, data);
        }
    }

    /// Get items from storage
    pub fn get(
        &self,
        extension_id: &str,
        area: StorageArea,
        keys: Option<Vec<String>>,
    ) -> Result<HashMap<String, Value>, StorageError> {
        self.ensure_loaded(extension_id, area);
        
        let cache = self.cache.read();
        let ext_cache = cache.get(extension_id)
            .and_then(|c| c.get(&area))
            .cloned()
            .unwrap_or_default();

        match keys {
            None => Ok(ext_cache),
            Some(keys) => {
                let mut result = HashMap::new();
                for key in keys {
                    if let Some(value) = ext_cache.get(&key) {
                        result.insert(key, value.clone());
                    }
                }
                Ok(result)
            }
        }
    }

    /// Set items in storage
    pub fn set(
        &self,
        extension_id: &str,
        area: StorageArea,
        items: HashMap<String, Value>,
    ) -> Result<HashMap<String, StorageChange>, StorageError> {
        self.ensure_loaded(extension_id, area);

        // Check quota
        self.check_quota(extension_id, area, &items)?;

        let mut changes = HashMap::new();
        
        {
            let mut cache = self.cache.write();
            let ext_cache = cache
                .get_mut(extension_id)
                .unwrap()
                .get_mut(&area)
                .unwrap();

            for (key, new_value) in items {
                let old_value = ext_cache.insert(key.clone(), new_value.clone());
                changes.insert(key, StorageChange {
                    old_value,
                    new_value: Some(new_value),
                });
            }
        }

        // Persist to disk
        let cache = self.cache.read();
        if let Some(ext_cache) = cache.get(extension_id).and_then(|c| c.get(&area)) {
            self.save_storage(extension_id, area, ext_cache)?;
        }

        Ok(changes)
    }

    /// Remove items from storage
    pub fn remove(
        &self,
        extension_id: &str,
        area: StorageArea,
        keys: Vec<String>,
    ) -> Result<HashMap<String, StorageChange>, StorageError> {
        self.ensure_loaded(extension_id, area);

        let mut changes = HashMap::new();

        {
            let mut cache = self.cache.write();
            let ext_cache = cache
                .get_mut(extension_id)
                .unwrap()
                .get_mut(&area)
                .unwrap();

            for key in keys {
                if let Some(old_value) = ext_cache.remove(&key) {
                    changes.insert(key, StorageChange {
                        old_value: Some(old_value),
                        new_value: None,
                    });
                }
            }
        }

        // Persist to disk
        let cache = self.cache.read();
        if let Some(ext_cache) = cache.get(extension_id).and_then(|c| c.get(&area)) {
            self.save_storage(extension_id, area, ext_cache)?;
        }

        Ok(changes)
    }

    /// Clear all items in storage
    pub fn clear(
        &self,
        extension_id: &str,
        area: StorageArea,
    ) -> Result<HashMap<String, StorageChange>, StorageError> {
        self.ensure_loaded(extension_id, area);

        let changes: HashMap<String, StorageChange>;

        {
            let mut cache = self.cache.write();
            let ext_cache = cache
                .get_mut(extension_id)
                .unwrap()
                .get_mut(&area)
                .unwrap();

            changes = ext_cache
                .drain()
                .map(|(k, v)| (k, StorageChange {
                    old_value: Some(v),
                    new_value: None,
                }))
                .collect();
        }

        // Persist to disk
        self.save_storage(extension_id, area, &HashMap::new())?;

        Ok(changes)
    }

    /// Get bytes in use
    pub fn get_bytes_in_use(
        &self,
        extension_id: &str,
        area: StorageArea,
        keys: Option<Vec<String>>,
    ) -> Result<usize, StorageError> {
        self.ensure_loaded(extension_id, area);

        let cache = self.cache.read();
        let ext_cache = cache.get(extension_id)
            .and_then(|c| c.get(&area))
            .cloned()
            .unwrap_or_default();

        let items_to_count: Vec<(&String, &Value)> = match &keys {
            None => ext_cache.iter().collect(),
            Some(keys) => ext_cache
                .iter()
                .filter(|(k, _)| keys.contains(k))
                .collect(),
        };

        let bytes: usize = items_to_count
            .iter()
            .map(|(k, v)| {
                k.len() + serde_json::to_string(v)
                    .map(|s| s.len())
                    .unwrap_or(0)
            })
            .sum();

        Ok(bytes)
    }

    /// Check if items fit within quota
    fn check_quota(
        &self,
        extension_id: &str,
        area: StorageArea,
        new_items: &HashMap<String, Value>,
    ) -> Result<(), StorageError> {
        let quota = match area {
            StorageArea::Local => QUOTA_BYTES_LOCAL,
            StorageArea::Sync => QUOTA_BYTES_SYNC,
            StorageArea::Session => QUOTA_BYTES_SESSION,
            StorageArea::Managed => return Err(StorageError::ReadOnly),
        };

        let current = self.get_bytes_in_use(extension_id, area, None)?;
        
        let new_bytes: usize = new_items
            .iter()
            .map(|(k, v)| {
                k.len() + serde_json::to_string(v)
                    .map(|s| s.len())
                    .unwrap_or(0)
            })
            .sum();

        if current + new_bytes > quota {
            return Err(StorageError::QuotaExceeded {
                quota,
                used: current + new_bytes,
            });
        }

        // For sync storage, also check item limits
        if area == StorageArea::Sync {
            let cache = self.cache.read();
            let item_count = cache
                .get(extension_id)
                .and_then(|c| c.get(&area))
                .map(|m| m.len())
                .unwrap_or(0);

            if item_count + new_items.len() > MAX_ITEMS_SYNC {
                return Err(StorageError::MaxItemsExceeded {
                    max: MAX_ITEMS_SYNC,
                    count: item_count + new_items.len(),
                });
            }

            // Check per-item size limit
            for (key, value) in new_items {
                let item_size = key.len() + serde_json::to_string(value)
                    .map(|s| s.len())
                    .unwrap_or(0);
                
                if item_size > QUOTA_BYTES_PER_ITEM_SYNC {
                    return Err(StorageError::ItemTooLarge {
                        key: key.clone(),
                        size: item_size,
                        max: QUOTA_BYTES_PER_ITEM_SYNC,
                    });
                }
            }
        }

        Ok(())
    }

    /// Clear all session storage (called on browser close)
    pub fn clear_session_storage(&self) {
        let mut cache = self.cache.write();
        for ext_cache in cache.values_mut() {
            ext_cache.remove(&StorageArea::Session);
        }
    }
}

/// Storage API errors
#[derive(Debug)]
pub enum StorageError {
    IoError(String),
    SerializeError(String),
    ReadOnly,
    QuotaExceeded { quota: usize, used: usize },
    MaxItemsExceeded { max: usize, count: usize },
    ItemTooLarge { key: String, size: usize, max: usize },
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(e) => write!(f, "IO error: {}", e),
            StorageError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            StorageError::ReadOnly => write!(f, "Storage is read-only"),
            StorageError::QuotaExceeded { quota, used } => {
                write!(f, "Quota exceeded: {} bytes used, {} allowed", used, quota)
            }
            StorageError::MaxItemsExceeded { max, count } => {
                write!(f, "Max items exceeded: {} items, {} allowed", count, max)
            }
            StorageError::ItemTooLarge { key, size, max } => {
                write!(f, "Item '{}' too large: {} bytes, {} allowed", key, size, max)
            }
        }
    }
}

impl std::error::Error for StorageError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_basic() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ExtensionStorage::new(temp_dir.path().to_path_buf());

        let mut items = HashMap::new();
        items.insert("key1".to_string(), serde_json::json!("value1"));
        items.insert("key2".to_string(), serde_json::json!(42));

        storage.set("test-ext", StorageArea::Local, items).unwrap();

        let result = storage.get("test-ext", StorageArea::Local, None).unwrap();
        assert_eq!(result.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(result.get("key2"), Some(&serde_json::json!(42)));
    }

    #[test]
    fn test_storage_remove() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ExtensionStorage::new(temp_dir.path().to_path_buf());

        let mut items = HashMap::new();
        items.insert("key1".to_string(), serde_json::json!("value1"));
        storage.set("test-ext", StorageArea::Local, items).unwrap();

        storage.remove("test-ext", StorageArea::Local, vec!["key1".to_string()]).unwrap();

        let result = storage.get("test-ext", StorageArea::Local, None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_storage_clear() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ExtensionStorage::new(temp_dir.path().to_path_buf());

        let mut items = HashMap::new();
        items.insert("key1".to_string(), serde_json::json!("value1"));
        items.insert("key2".to_string(), serde_json::json!("value2"));
        storage.set("test-ext", StorageArea::Local, items).unwrap();

        storage.clear("test-ext", StorageArea::Local).unwrap();

        let result = storage.get("test-ext", StorageArea::Local, None).unwrap();
        assert!(result.is_empty());
    }
}
