use crate::storage::StorageBackend;
use redis::{Client, Commands, Connection};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Error types for Redis storage operations
#[derive(Debug, Error)]
pub enum RedisStorageError {
    #[error("Redis connection error: {0}")]
    Connection(#[from] redis::RedisError),
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),
    #[error("Lock error: {0}")]
    Lock(String),
}

/// Redis-based storage backend that implements StorageBackend trait
pub struct RedisStorage {
    connection: Arc<Mutex<Connection>>,
    key_prefix: String,
}

impl RedisStorage {
    /// Create a new Redis storage with the given connection URL and default prefix
    pub fn new(redis_url: &str) -> Result<Self, RedisStorageError> {
        Self::new_with_prefix(redis_url, "pocketflow")
    }

    /// Create a new Redis storage with the given connection URL and key prefix
    pub fn new_with_prefix(redis_url: &str, key_prefix: &str) -> Result<Self, RedisStorageError> {
        let client = Client::open(redis_url)?;
        let connection = client.get_connection()?;
        
        Ok(RedisStorage {
            connection: Arc::new(Mutex::new(connection)),
            key_prefix: key_prefix.to_string(),
        })
    }

    /// Get the full Redis key by adding the prefix
    fn get_full_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    /// Remove the prefix from a Redis key to get the original key
    fn remove_prefix(&self, full_key: &str) -> Option<String> {
        let prefix_with_colon = format!("{}:", self.key_prefix);
        if full_key.starts_with(&prefix_with_colon) {
            Some(full_key[prefix_with_colon.len()..].to_string())
        } else {
            None
        }
    }

    /// Helper to execute a command with proper error handling
    fn with_connection<F, R>(&self, f: F) -> Result<R, RedisStorageError>
    where
        F: FnOnce(&mut Connection) -> Result<R, redis::RedisError>,
    {
        let mut conn = self.connection
            .lock()
            .map_err(|e| RedisStorageError::Lock(e.to_string()))?;
        f(&mut *conn).map_err(RedisStorageError::Connection)
    }
}

impl StorageBackend for RedisStorage {
    type Error = RedisStorageError;

    fn set(&mut self, key: String, value: Value) -> Result<(), Self::Error> {
        let full_key = self.get_full_key(&key);
        let json_string = serde_json::to_string(&value)?;
        
        self.with_connection(|conn| {
            let _: () = conn.set(&full_key, &json_string)?;
            Ok(())
        })
    }

    fn get(&self, key: &str) -> Result<Option<Value>, Self::Error> {
        let full_key = self.get_full_key(key);
        
        self.with_connection(|conn| {
            let result: Option<String> = conn.get(&full_key)?;
            
            match result {
                Some(json_string) => {
                    let value: Value = serde_json::from_str(&json_string)
                        .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "JSON parse error", e.to_string())))?;
                    Ok(Some(value))
                }
                None => Ok(None),
            }
        })
    }

    fn remove(&mut self, key: &str) -> Result<Option<Value>, Self::Error> {
        let full_key = self.get_full_key(key);
        
        // First try to get the existing value
        let existing_value = self.get(key)?;
        
        // Then delete the key  
        self.with_connection(|conn| {
            let _: u32 = conn.del(&full_key)?;
            Ok(())
        })?;
        
        Ok(existing_value)
    }

    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        let full_key = self.get_full_key(key);
        
        self.with_connection(|conn| {
            let result: bool = conn.exists(&full_key)?;
            Ok(result)
        })
    }

    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        let pattern = format!("{}:*", self.key_prefix);
        
        self.with_connection(|conn| {
            let full_keys: Vec<String> = conn.keys(&pattern)?;
            
            let keys = full_keys
                .into_iter()
                .filter_map(|full_key| self.remove_prefix(&full_key))
                .collect();
                
            Ok(keys)
        })
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        let pattern = format!("{}:*", self.key_prefix);
        
        self.with_connection(|conn| {
            let full_keys: Vec<String> = conn.keys(&pattern)?;
            
            if !full_keys.is_empty() {
                let _: u32 = conn.del(&full_keys)?;
            }
            
            Ok(())
        })
    }

    fn len(&self) -> Result<usize, Self::Error> {
        let pattern = format!("{}:*", self.key_prefix);
        
        self.with_connection(|conn| {
            let full_keys: Vec<String> = conn.keys(&pattern)?;
            Ok(full_keys.len())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Note: These tests require a Redis server running on localhost:6379
    // Run: docker run --rm -p 6379:6379 redis:latest
    
    fn setup_redis() -> Result<RedisStorage, RedisStorageError> {
        RedisStorage::new_with_prefix("redis://127.0.0.1:6379/", "pocketflow_test")
    }

    #[test] 
    #[ignore] // Requires Redis server
    fn test_redis_storage_basic_operations() -> Result<(), RedisStorageError> {
        let mut storage = setup_redis()?;
        
        // Clear any existing test data
        storage.clear()?;
        
        // Test set and get
        storage.set("test_key".to_string(), json!("test_value"))?;
        let value = storage.get("test_key")?;
        assert_eq!(value, Some(json!("test_value")));
        
        // Test contains_key
        assert!(storage.contains_key("test_key")?);
        assert!(!storage.contains_key("nonexistent_key")?);
        
        // Test remove
        let removed = storage.remove("test_key")?;
        assert_eq!(removed, Some(json!("test_value")));
        assert!(!storage.contains_key("test_key")?);
        
        Ok(())
    }

    #[test]
    #[ignore] // Requires Redis server
    fn test_redis_storage_complex_data() -> Result<(), RedisStorageError> {
        let mut storage = setup_redis()?;
        storage.clear()?;
        
        let complex_data = json!({
            "user_id": 123,
            "preferences": {
                "theme": "dark",
                "language": "en"
            },
            "tags": ["rust", "redis", "pocketflow"]
        });
        
        storage.set("user_data".to_string(), complex_data.clone())?;
        let retrieved = storage.get("user_data")?;
        assert_eq!(retrieved, Some(complex_data));
        
        Ok(())
    }

    #[test]
    #[ignore] // Requires Redis server  
    fn test_redis_storage_keys_and_len() -> Result<(), RedisStorageError> {
        let mut storage = setup_redis()?;
        storage.clear()?;
        
        // Add several keys
        storage.set("key1".to_string(), json!("value1"))?;
        storage.set("key2".to_string(), json!("value2"))?;
        storage.set("key3".to_string(), json!("value3"))?;
        
        // Test len
        assert_eq!(storage.len()?, 3);
        assert!(!storage.is_empty()?);
        
        // Test keys
        let mut keys = storage.keys()?;
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2", "key3"]);
        
        // Test clear
        storage.clear()?;
        assert_eq!(storage.len()?, 0);
        assert!(storage.is_empty()?);
        
        Ok(())
    }
}