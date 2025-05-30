mod memory;
mod file;

pub use memory::{InMemoryStorage, InMemoryError};
pub use file::{FileStorage, FileStorageError};
use serde_json::Value;
use std::error::Error;

/// Trait defining the interface for storage backends used by SharedStore
pub trait StorageBackend: Send + Sync {
    /// Error type returned by storage operations
    type Error: Error + Send + Sync + 'static;

    /// Store a value with the given key
    fn set(&mut self, key: String, value: Value) -> Result<(), Self::Error>;

    /// Retrieve a value by key
    fn get(&self, key: &str) -> Result<Option<Value>, Self::Error>;

    /// Remove a value by key, returning it if it existed
    fn remove(&mut self, key: &str) -> Result<Option<Value>, Self::Error>;

    /// Check if a key exists
    fn contains_key(&self, key: &str) -> Result<bool, Self::Error>;

    /// Get all keys
    fn keys(&self) -> Result<Vec<String>, Self::Error>;

    /// Clear all data
    fn clear(&mut self) -> Result<(), Self::Error>;

    /// Get the number of stored items
    fn len(&self) -> Result<usize, Self::Error>;

    /// Check if the storage is empty
    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.len()? == 0)
    }
}

/// Async version of StorageBackend for I/O-bound operations
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncStorageBackend: Send + Sync {
    /// Error type returned by storage operations
    type Error: Error + Send + Sync + 'static;

    /// Store a value with the given key
    async fn set(&mut self, key: String, value: Value) -> Result<(), Self::Error>;

    /// Retrieve a value by key
    async fn get(&self, key: &str) -> Result<Option<Value>, Self::Error>;

    /// Remove a value by key, returning it if it existed
    async fn remove(&mut self, key: &str) -> Result<Option<Value>, Self::Error>;

    /// Check if a key exists
    async fn contains_key(&self, key: &str) -> Result<bool, Self::Error>;

    /// Get all keys
    async fn keys(&self) -> Result<Vec<String>, Self::Error>;

    /// Clear all data
    async fn clear(&mut self) -> Result<(), Self::Error>;

    /// Get the number of stored items
    async fn len(&self) -> Result<usize, Self::Error>;

    /// Check if the storage is empty
    async fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.len().await? == 0)
    }
}