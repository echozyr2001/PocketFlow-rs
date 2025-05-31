use crate::storage::{InMemoryStorage, StorageBackend};
use serde_json::Value;

/// SharedStore provides a type-safe interface for data communication between nodes
/// in PocketFlow workflows. It can use different storage backends for flexibility.
#[derive(Debug)]
pub struct SharedStore<S: StorageBackend> {
    storage: S,
}

/// Type alias for the default in-memory SharedStore
pub type InMemorySharedStore = SharedStore<InMemoryStorage>;

impl<S: StorageBackend> SharedStore<S> {
    /// Creates a new SharedStore with the provided storage backend
    pub fn with_storage(storage: S) -> Self {
        Self { storage }
    }

    /// Sets a value in the SharedStore.
    ///
    /// # Arguments
    ///
    /// * `key` - The key (String) to associate with the value.
    /// * `value` - The `serde_json::Value` to store.
    pub fn set(&mut self, key: String, value: Value) -> Result<(), S::Error> {
        self.storage.set(key, value)
    }

    /// Gets a value from the SharedStore.
    ///
    /// # Arguments
    ///
    /// * `key` - The key (String) of the value to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result<Option<Value>, S::Error>` which is `Ok(Some(Value))` if the key exists,
    /// `Ok(None)` if it doesn't, or `Err` if there was a storage error.
    pub fn get(&self, key: &str) -> Result<Option<Value>, S::Error> {
        self.storage.get(key)
    }

    /// Removes a value from the SharedStore, returning it if it existed.
    ///
    /// # Arguments
    ///
    /// * `key` - The key (String) of the value to remove.
    ///
    /// # Returns
    ///
    /// A `Result<Option<Value>, S::Error>` which is `Ok(Some(Value))` if the key existed,
    /// `Ok(None)` if it didn't, or `Err` if there was a storage error.
    pub fn remove(&mut self, key: &str) -> Result<Option<Value>, S::Error> {
        self.storage.remove(key)
    }

    /// Checks if a key exists in the SharedStore.
    pub fn contains_key(&self, key: &str) -> Result<bool, S::Error> {
        self.storage.contains_key(key)
    }

    /// Gets all keys from the SharedStore.
    pub fn keys(&self) -> Result<Vec<String>, S::Error> {
        self.storage.keys()
    }

    /// Clears all data from the SharedStore.
    pub fn clear(&mut self) -> Result<(), S::Error> {
        self.storage.clear()
    }

    /// Gets the number of items in the SharedStore.
    pub fn len(&self) -> Result<usize, S::Error> {
        self.storage.len()
    }

    /// Checks if the SharedStore is empty.
    pub fn is_empty(&self) -> Result<bool, S::Error> {
        self.storage.is_empty()
    }

    /// Convenience method to set a serializable value
    pub fn set_serializable<T: serde::Serialize>(
        &mut self,
        key: String,
        value: T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json_value = serde_json::to_value(value)?;
        self.storage
            .set(key, json_value)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Convenience method to get and deserialize a value
    pub fn get_deserializable<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>> {
        match self.storage.get(key) {
            Ok(Some(value)) => {
                let deserialized = serde_json::from_value(value)?;
                Ok(Some(deserialized))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }
}

// Convenience constructors for common storage backends
impl InMemorySharedStore {
    /// Creates a new SharedStore with in-memory storage
    pub fn new() -> Self {
        Self::with_storage(InMemoryStorage::new())
    }

    /// Creates a new SharedStore with in-memory storage and specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_storage(InMemoryStorage::with_capacity(capacity))
    }
}

impl Default for InMemorySharedStore {
    fn default() -> Self {
        Self::new()
    }
}

// Basic tests for SharedStore
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "storage-file")]
    use crate::storage::FileStorage;
    use serde_json::json;
    #[cfg(feature = "storage-file")]
    use tempfile::tempdir;

    #[test]
    fn test_in_memory_shared_store_set_get() {
        let mut store = InMemorySharedStore::new();
        let key = "test_key".to_string();
        let value = json!({"message": "hello"});

        store.set(key.clone(), value.clone()).unwrap();

        assert_eq!(store.get(&key).unwrap(), Some(value));
    }

    #[test]
    fn test_in_memory_shared_store_get_non_existent() {
        let store = InMemorySharedStore::new();
        assert_eq!(store.get("non_existent_key").unwrap(), None);
    }

    #[test]
    fn test_in_memory_shared_store_remove() {
        let mut store = InMemorySharedStore::new();
        let key = "test_key".to_string();
        let value = json!("test_value");

        store.set(key.clone(), value.clone()).unwrap();
        assert_eq!(store.remove(&key).unwrap(), Some(value));
        assert_eq!(store.get(&key).unwrap(), None);
    }

    #[test]
    fn test_in_memory_shared_store_set_get_serializable() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
        struct MyStruct {
            id: i32,
            name: String,
        }

        let mut store = InMemorySharedStore::new();
        let my_data = MyStruct {
            id: 1,
            name: "PocketFlow".to_string(),
        };

        store
            .set_serializable("my_data".to_string(), my_data.clone())
            .unwrap();

        let retrieved_my_data: MyStruct = store.get_deserializable("my_data").unwrap().unwrap();
        assert_eq!(retrieved_my_data, my_data);
    }

    #[test]
    fn test_shared_store_additional_methods() {
        let mut store = InMemorySharedStore::new();

        // Test empty store
        assert!(store.is_empty().unwrap());
        assert_eq!(store.len().unwrap(), 0);
        assert!(store.keys().unwrap().is_empty());

        // Add some data
        store.set("key1".to_string(), json!("value1")).unwrap();
        store.set("key2".to_string(), json!("value2")).unwrap();

        // Test non-empty store
        assert!(!store.is_empty().unwrap());
        assert_eq!(store.len().unwrap(), 2);
        assert!(store.contains_key("key1").unwrap());
        assert!(!store.contains_key("key3").unwrap());

        let mut keys = store.keys().unwrap();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2"]);

        // Test clear
        store.clear().unwrap();
        assert!(store.is_empty().unwrap());
        assert_eq!(store.len().unwrap(), 0);
    }

    #[cfg(feature = "storage-file")]
    #[test]
    fn test_file_shared_store() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_shared_store.json");

        let file_storage = FileStorage::new(&file_path).unwrap();
        let mut store = SharedStore::with_storage(file_storage);

        // Test basic operations
        store
            .set("file_key".to_string(), json!("file_value"))
            .unwrap();
        assert_eq!(store.get("file_key").unwrap(), Some(json!("file_value")));

        // Test persistence by creating a new store with the same file
        let file_storage2 = FileStorage::new(&file_path).unwrap();
        let store2 = SharedStore::with_storage(file_storage2);
        assert_eq!(store2.get("file_key").unwrap(), Some(json!("file_value")));
    }
}