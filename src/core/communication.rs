use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

// --- StoreBackend Trait and Implementations ---

/// Type alias for the values stored in the backend.
/// `Arc` allows for cheap cloning when retrieving, and `dyn Any` allows storing any type.
pub type StoredValue = Arc<dyn Any + Send + Sync>;

/// Trait defining the capabilities of a storage backend for `SharedStore`.
pub trait StoreBackend: Send + Sync {
    /// Inserts a value into the store.
    fn insert(&self, key: &str, value: StoredValue);

    /// Retrieves a value from the store.
    /// Returns a clone of the `Arc` if the key exists.
    fn get(&self, key: &str) -> Option<StoredValue>;

    /// Removes a value from the store, returning it if it existed.
    fn remove(&self, key: &str) -> Option<StoredValue>;

    /// Checks if a key exists in the store.
    fn contains_key(&self, key: &str) -> bool;

    /// Returns a list of all keys in the store.
    fn keys(&self) -> Vec<String>;
}

/// Default in-memory backend for `SharedStore`.
#[derive(Default)]
pub struct InMemoryBackend {
    inner: RwLock<HashMap<String, StoredValue>>,
}

impl InMemoryBackend {
    /// Creates a new, empty `InMemoryBackend`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl StoreBackend for InMemoryBackend {
    fn insert(&self, key: &str, value: StoredValue) {
        self.inner.write().insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<StoredValue> {
        self.inner.read().get(key).cloned() // .cloned() clones the Arc
    }

    fn remove(&self, key: &str) -> Option<StoredValue> {
        self.inner.write().remove(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.inner.read().contains_key(key)
    }

    fn keys(&self) -> Vec<String> {
        self.inner.read().keys().cloned().collect()
    }
}

// --- SharedStore ---

/// `SharedStore` provides global data sharing between nodes, backed by a `StoreBackend`.
/// It allows for storing and retrieving any `'static + Send + Sync` type.
#[derive(Clone)]
pub struct SharedStore {
    backend: Arc<dyn StoreBackend>,
}

impl SharedStore {
    /// Creates a new `SharedStore` with the given backend.
    pub fn new(backend: Arc<dyn StoreBackend>) -> Self {
        Self { backend }
    }

    /// Creates a new `SharedStore` with a default `InMemoryBackend`.
    pub fn new_in_memory() -> Self {
        Self {
            backend: Arc::new(InMemoryBackend::new()),
        }
    }

    /// Inserts a value into the store. The value is wrapped in an `Arc<dyn Any + Send + Sync>`.
    ///
    /// # Examples
    /// ```
    /// # use pocketflow_rs::communication::SharedStore;
    /// # use serde_json::json;
    /// let store = SharedStore::new_in_memory();
    /// store.insert("my_key", 42);
    /// store.insert("json_val", json!({"data": "example"}));
    /// ```
    pub fn insert<T: 'static + Send + Sync>(&self, key: &str, value: T) {
        self.backend.insert(key, Arc::new(value));
    }

    /// Retrieves a clone of a value from the store, downcasting it to type `T`.
    ///
    /// `T` must implement `Clone`.
    ///
    /// Returns `None` if the key does not exist or if the stored value is not of type `T`.
    ///
    /// # Examples
    /// ```
    /// # use pocketflow_rs::communication::SharedStore;
    /// # use serde_json::{json, Value as JsonValue};
    /// let store = SharedStore::new_in_memory();
    /// store.insert("my_num", 100);
    /// store.insert("my_json", json!("hello"));
    ///
    /// assert_eq!(store.get::<i32>("my_num"), Some(100));
    /// assert_eq!(store.get::<JsonValue>("my_json"), Some(json!("hello")));
    /// assert_eq!(store.get::<String>("my_num"), None); // Stored as i32, not String
    /// ```
    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        let retrieved_arc: Option<StoredValue> = self.backend.get(key);
        match retrieved_arc {
            Some(arc_val) => {
                // arc_val is Arc<dyn Any + Send + Sync>
                // Attempt to downcast the reference from the Arc's content
                let downcast_result: Option<&T> = arc_val.downcast_ref::<T>();
                downcast_result.cloned()
            }
            None => None, // Key not found
        }
    }

    /// Removes a value from the store.
    /// Returns the `Arc`'d value if the key existed.
    pub fn remove(&self, key: &str) -> Option<StoredValue> {
        self.backend.remove(key)
    }

    /// Checks if a key exists in the store.
    pub fn contains_key(&self, key: &str) -> bool {
        self.backend.contains_key(key)
    }

    /// Returns a list of all keys in the store.
    pub fn keys(&self) -> Vec<String> {
        self.backend.keys()
    }
}

// --- Params and ParamsBuilder ---
// These remain unchanged for now, as they serve a different purpose (node configuration)
// and their reliance on JsonValue is generally acceptable for that role.
// Consider moving them to a separate module like `src/core/params.rs` in the future.

/// `Params` represents immutable configuration for nodes.
/// Similar to stack memory - passed down from parent to child.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Params {
    inner: HashMap<String, JsonValue>,
}

impl Params {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Create params from a HashMap.
    pub fn from_map(map: HashMap<String, JsonValue>) -> Self {
        Self { inner: map }
    }

    /// Set a parameter value.
    pub fn set<V: Serialize>(&mut self, key: &str, value: V) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.inner.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Get a parameter value and deserialize to type `T`.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inner
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a reference to the raw `JsonValue`.
    pub fn get_value(&self, key: &str) -> Option<&JsonValue> {
        self.inner.get(key)
    }

    /// Creates a new `ParamsBuilder` instance.
    pub fn builder() -> ParamsBuilder {
        ParamsBuilder::new()
    }

    /// Merge with another `Params` instance.
    /// Used when combining parent and child params.
    pub fn merge(&self, other: &Params) -> Self {
        let mut new_map = self.inner.clone();
        new_map.extend(other.inner.clone());
        Self { inner: new_map }
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }
}

/// `ParamsBuilder` provides a fluent interface for constructing `Params` instances.
#[derive(Default)]
pub struct ParamsBuilder {
    inner: HashMap<String, JsonValue>,
}

impl ParamsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a key-value pair into the parameters.
    ///
    /// The value will be serialized to a `JsonValue`.
    /// Returns an error if serialization fails.
    pub fn insert<V: Serialize>(mut self, key: &str, value: V) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.inner.insert(key.to_string(), json_value);
        Ok(self)
    }

    /// Inserts a key-value pair into the parameters, panicking if serialization fails.
    ///
    /// The value will be serialized to a `JsonValue`.
    /// Use this method when you are certain that serialization will succeed.
    pub fn insert_unwrap<V: Serialize>(mut self, key: &str, value: V) -> Self {
        let json_value = serde_json::to_value(value)
            .expect("ParamsBuilder: Failed to serialize value during insert_unwrap");
        self.inner.insert(key.to_string(), json_value);
        self
    }

    /// Builds the `Params` instance from the accumulated parameters.
    pub fn build(self) -> Params {
        Params { inner: self.inner }
    }
}

/// `ParamsContainer` provides params management for nodes and flows.
pub trait ParamsContainer {
    /// Set params for this container.
    fn set_params(&mut self, params: Params);

    /// Get current params.
    fn get_params(&self) -> &Params;

    /// Get mutable reference to params.
    fn get_params_mut(&mut self) -> &mut Params;
}
