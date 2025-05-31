use super::StorageBackend;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Simple in-memory storage backend using HashMap
#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    data: HashMap<String, Value>,
}

/// Error type for in-memory storage operations
#[derive(Debug, Clone)]
pub enum InMemoryStorageError {
    /// This implementation doesn't actually produce errors, but we need an error type
    /// for trait compatibility
    Never,
}

impl fmt::Display for InMemoryStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InMemoryStorageError::Never => write!(f, "This error should never occur"),
        }
    }
}

impl std::error::Error for InMemoryStorageError {}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create a new in-memory storage with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }
}

impl StorageBackend for InMemoryStorage {
    type Error = InMemoryStorageError;

    fn set(&mut self, key: String, value: Value) -> Result<(), Self::Error> {
        self.data.insert(key, value);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Value>, Self::Error> {
        Ok(self.data.get(key).cloned())
    }

    fn remove(&mut self, key: &str) -> Result<Option<Value>, Self::Error> {
        Ok(self.data.remove(key))
    }

    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(key))
    }

    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.data.clear();
        Ok(())
    }

    fn len(&self) -> Result<usize, Self::Error> {
        Ok(self.data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_in_memory_storage_basic_operations() {
        let mut storage = InMemoryStorage::new();

        // Test set and get
        storage.set("key1".to_string(), json!("value1")).unwrap();
        assert_eq!(storage.get("key1").unwrap(), Some(json!("value1")));

        // Test non-existent key
        assert_eq!(storage.get("nonexistent").unwrap(), None);

        // Test contains_key
        assert!(storage.contains_key("key1").unwrap());
        assert!(!storage.contains_key("nonexistent").unwrap());

        // Test len
        assert_eq!(storage.len().unwrap(), 1);
        assert!(!storage.is_empty().unwrap());

        // Test remove
        assert_eq!(storage.remove("key1").unwrap(), Some(json!("value1")));
        assert_eq!(storage.remove("key1").unwrap(), None);
        assert_eq!(storage.len().unwrap(), 0);
        assert!(storage.is_empty().unwrap());
    }

    #[test]
    fn test_in_memory_storage_keys_and_clear() {
        let mut storage = InMemoryStorage::new();

        storage.set("key1".to_string(), json!("value1")).unwrap();
        storage.set("key2".to_string(), json!("value2")).unwrap();
        storage.set("key3".to_string(), json!("value3")).unwrap();

        let mut keys = storage.keys().unwrap();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2", "key3"]);

        storage.clear().unwrap();
        assert_eq!(storage.len().unwrap(), 0);
        assert!(storage.keys().unwrap().is_empty());
    }
}
