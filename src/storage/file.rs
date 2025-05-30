use super::StorageBackend;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// File-based storage backend that persists data to JSON files
#[derive(Debug, Clone)]
pub struct FileStorage {
    file_path: PathBuf,
    data: HashMap<String, Value>,
}

/// Error type for file storage operations
#[derive(Debug)]
pub enum FileStorageError {
    /// I/O error
    Io(io::Error),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
}

impl std::fmt::Display for FileStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileStorageError::Io(e) => write!(f, "I/O error: {}", e),
            FileStorageError::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for FileStorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FileStorageError::Io(e) => Some(e),
            FileStorageError::Json(e) => Some(e),
        }
    }
}

impl From<io::Error> for FileStorageError {
    fn from(error: io::Error) -> Self {
        FileStorageError::Io(error)
    }
}

impl From<serde_json::Error> for FileStorageError {
    fn from(error: serde_json::Error) -> Self {
        FileStorageError::Json(error)
    }
}

impl FileStorage {
    /// Create a new file storage with the specified file path
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, FileStorageError> {
        let file_path = file_path.as_ref().to_path_buf();
        let data = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            if content.trim().is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(&content)?
            }
        } else {
            HashMap::new()
        };

        Ok(Self { file_path, data })
    }

    /// Save the current data to file
    fn save_to_file(&self) -> Result<(), FileStorageError> {
        let json_data = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, json_data)?;
        Ok(())
    }
}

impl StorageBackend for FileStorage {
    type Error = FileStorageError;

    fn set(&mut self, key: String, value: Value) -> Result<(), Self::Error> {
        self.data.insert(key, value);
        self.save_to_file()
    }

    fn get(&self, key: &str) -> Result<Option<Value>, Self::Error> {
        Ok(self.data.get(key).cloned())
    }

    fn remove(&mut self, key: &str) -> Result<Option<Value>, Self::Error> {
        let result = self.data.remove(key);
        self.save_to_file()?;
        Ok(result)
    }

    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(key))
    }

    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.data.clear();
        self.save_to_file()
    }

    fn len(&self) -> Result<usize, Self::Error> {
        Ok(self.data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_storage_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_storage.json");
        
        let mut storage = FileStorage::new(&file_path).unwrap();
        
        // Test set and get
        storage.set("key1".to_string(), json!("value1")).unwrap();
        assert_eq!(storage.get("key1").unwrap(), Some(json!("value1")));
        
        // Test persistence by creating a new instance
        let storage2 = FileStorage::new(&file_path).unwrap();
        assert_eq!(storage2.get("key1").unwrap(), Some(json!("value1")));
        
        // Clean up
        fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_storage_persistence() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_persistence.json");
        
        // Create storage and add data
        {
            let mut storage = FileStorage::new(&file_path).unwrap();
            storage.set("persistent_key".to_string(), json!({"data": "persistent_value"})).unwrap();
        }
        
        // Create new storage instance and verify data persisted
        {
            let storage = FileStorage::new(&file_path).unwrap();
            assert_eq!(
                storage.get("persistent_key").unwrap(),
                Some(json!({"data": "persistent_value"}))
            );
            assert_eq!(storage.len().unwrap(), 1);
        }
        
        // Clean up
        fs::remove_file(&file_path).ok();
    }
}