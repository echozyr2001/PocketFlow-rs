use crate::storage::AsyncStorageBackend;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

/// An async version of SharedStore for use with AsyncStorageBackend implementations
pub struct AsyncSharedStore<S: AsyncStorageBackend> {
    storage: Arc<Mutex<S>>,
}

impl<S: AsyncStorageBackend> AsyncSharedStore<S> {
    /// Create a new async shared store with the given storage backend
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
        }
    }

    /// Store a value with the given key
    pub async fn set(&self, key: String, value: Value) -> Result<(), S::Error> {
        let mut storage = self.storage.lock().await;
        storage.set(key, value).await
    }

    /// Retrieve a value by key
    pub async fn get(&self, key: &str) -> Result<Option<Value>, S::Error> {
        let storage = self.storage.lock().await;
        storage.get(key).await
    }

    /// Remove a value by key, returning it if it existed
    pub async fn remove(&self, key: &str) -> Result<Option<Value>, S::Error> {
        let mut storage = self.storage.lock().await;
        storage.remove(key).await
    }

    /// Check if a key exists
    pub async fn contains_key(&self, key: &str) -> Result<bool, S::Error> {
        let storage = self.storage.lock().await;
        storage.contains_key(key).await
    }

    /// Get all keys
    pub async fn keys(&self) -> Result<Vec<String>, S::Error> {
        let storage = self.storage.lock().await;
        storage.keys().await
    }

    /// Clear all data
    pub async fn clear(&self) -> Result<(), S::Error> {
        let mut storage = self.storage.lock().await;
        storage.clear().await
    }

    /// Get the number of stored items
    pub async fn len(&self) -> Result<usize, S::Error> {
        let storage = self.storage.lock().await;
        storage.len().await
    }

    /// Check if the storage is empty
    pub async fn is_empty(&self) -> Result<bool, S::Error> {
        let storage = self.storage.lock().await;
        storage.is_empty().await
    }

    /// Store a serializable value (convenience method)
    pub async fn set_serializable<T>(
        &self,
        key: String,
        value: &T,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.set(key, json_value)
            .await
            .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;
        Ok(())
    }

    /// Retrieve and deserialize a value (convenience method)
    pub async fn get_deserializable<T>(
        &self,
        key: &str,
    ) -> Result<Option<T>, Box<dyn Error + Send + Sync>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(value) = self
            .get(key)
            .await
            .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
        {
            let deserialized: T = serde_json::from_value(value)?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    /// Get a mutable reference to the underlying storage (use with caution)
    pub async fn storage_mut(&self) -> tokio::sync::MutexGuard<'_, S> {
        self.storage.lock().await
    }

    /// Get a reference to the underlying storage (use with caution)
    pub fn storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }
}

impl<S: AsyncStorageBackend> Clone for AsyncSharedStore<S> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "storage-memory")]
    use crate::storage::InMemoryStorageError;
    use serde_json::json;
    use std::collections::HashMap;

    // Mock async storage for testing
    #[cfg(feature = "storage-memory")]
    struct MockAsyncStorage {
        data: HashMap<String, Value>,
    }

    #[cfg(feature = "storage-memory")]
    impl MockAsyncStorage {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
    }

    #[cfg(feature = "storage-memory")]
    #[async_trait::async_trait]
    impl AsyncStorageBackend for MockAsyncStorage {
        type Error = InMemoryStorageError;

        async fn set(&mut self, key: String, value: Value) -> Result<(), Self::Error> {
            self.data.insert(key, value);
            Ok(())
        }

        async fn get(&self, key: &str) -> Result<Option<Value>, Self::Error> {
            Ok(self.data.get(key).cloned())
        }

        async fn remove(&mut self, key: &str) -> Result<Option<Value>, Self::Error> {
            Ok(self.data.remove(key))
        }

        async fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
            Ok(self.data.contains_key(key))
        }

        async fn keys(&self) -> Result<Vec<String>, Self::Error> {
            Ok(self.data.keys().cloned().collect())
        }

        async fn clear(&mut self) -> Result<(), Self::Error> {
            self.data.clear();
            Ok(())
        }

        async fn len(&self) -> Result<usize, Self::Error> {
            Ok(self.data.len())
        }
    }

    #[cfg(feature = "storage-memory")]
    #[tokio::test]
    async fn test_async_shared_store() -> Result<(), Box<dyn Error + Send + Sync>> {
        let storage = MockAsyncStorage::new();
        let store = AsyncSharedStore::new(storage);

        // Test set and get
        store.set("test".to_string(), json!("value")).await?;
        let value = store.get("test").await?;
        assert_eq!(value, Some(json!("value")));

        // Test serialization
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            count: u32,
        }

        let test_data = TestData {
            name: "test".to_string(),
            count: 42,
        };

        store
            .set_serializable("struct_test".to_string(), &test_data)
            .await?;
        let retrieved: Option<TestData> = store.get_deserializable("struct_test").await?;
        assert_eq!(retrieved, Some(test_data));

        Ok(())
    }
}
