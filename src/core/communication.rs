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

/// Trait defining the capabilities of a storage backend for `BaseSharedStore`.
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

/// Default in-memory backend for `BaseSharedStore`.
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
        self.inner.read().get(key).cloned()
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

// --- SharedStore Trait (New) ---

/// Trait defining the data access interface for nodes.
/// This will be implemented by `BaseSharedStore`.
/// Methods operate on `StoredValue` to ensure trait is object-safe.
pub trait SharedStore: Send + Sync {
    /// Inserts an already boxed and Arc'd value into the store.
    fn insert_value(&self, key: &str, value: StoredValue);

    /// Retrieves an Arc'd value from the store. Downcasting is done by the caller.
    fn get_value(&self, key: &str) -> Option<StoredValue>;

    /// Removes a value from the store.
    /// Returns the `Arc`'d value if the key existed.
    fn remove_value(&self, key: &str) -> Option<StoredValue>;

    /// Checks if a key exists in the store.
    fn contains_key(&self, key: &str) -> bool;

    /// Returns a list of all keys in the store.
    fn keys(&self) -> Vec<String>;
}

// --- BaseSharedStore (Old SharedStore struct, renamed) ---

/// `BaseSharedStore` provides a concrete implementation of shared data storage,
/// backed by a `StoreBackend`. It implements the `SharedStore` trait.
#[derive(Clone)]
pub struct BaseSharedStore {
    backend: Arc<dyn StoreBackend>,
}

impl BaseSharedStore {
    /// Creates a new `BaseSharedStore` with the given backend.
    pub fn new(backend: Arc<dyn StoreBackend>) -> Self {
        Self { backend }
    }

    /// Creates a new `BaseSharedStore` with a default `InMemoryBackend`.
    pub fn new_in_memory() -> Self {
        Self {
            backend: Arc::new(InMemoryBackend::new()),
        }
    }
}

// BaseSharedStore still provides convenient generic methods for direct use.
impl BaseSharedStore {
    /// Inserts a value of any type `T` into the store.
    /// The value is wrapped in an `Arc<dyn Any + Send + Sync>`.
    pub fn insert<T: 'static + Send + Sync>(&self, key: &str, value: T) {
        self.backend.insert(key, Arc::new(value));
    }

    /// Retrieves a clone of a value of type `T` from the store.
    /// `T` must implement `Clone`.
    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        self.backend
            .get(key)
            .and_then(|arc_val| arc_val.downcast_ref::<T>().cloned())
    }
    // Note: The generic remove<T> is less common, so remove() returning StoredValue is fine.
}

impl SharedStore for BaseSharedStore {
    fn insert_value(&self, key: &str, value: StoredValue) {
        self.backend.insert(key, value);
    }

    fn get_value(&self, key: &str) -> Option<StoredValue> {
        self.backend.get(key)
    }

    fn remove_value(&self, key: &str) -> Option<StoredValue> {
        self.backend.remove(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        // This method is already non-generic in BaseSharedStore, can be called directly
        // or re-implemented here for clarity if BaseSharedStore's own methods were private.
        // For now, assuming BaseSharedStore's methods are public and can be reused.
        self.backend.contains_key(key)
    }

    fn keys(&self) -> Vec<String> {
        self.backend.keys()
    }
}

// --- Params and ParamsBuilder ---
// (Content remains the same)

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

    pub fn from_map(map: HashMap<String, JsonValue>) -> Self {
        Self { inner: map }
    }

    pub fn set<V: Serialize>(&mut self, key: &str, value: V) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.inner.insert(key.to_string(), json_value);
        Ok(())
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inner
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn get_value(&self, key: &str) -> Option<&JsonValue> {
        self.inner.get(key)
    }

    pub fn builder() -> ParamsBuilder {
        ParamsBuilder::new()
    }

    pub fn merge(&self, other: &Params) -> Self {
        let mut new_map = self.inner.clone();
        new_map.extend(other.inner.clone());
        Self { inner: new_map }
    }

    pub fn keys(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }
}

#[derive(Default)]
pub struct ParamsBuilder {
    inner: HashMap<String, JsonValue>,
}

impl ParamsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<V: Serialize>(mut self, key: &str, value: V) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.inner.insert(key.to_string(), json_value);
        Ok(self)
    }

    pub fn insert_unwrap<V: Serialize>(mut self, key: &str, value: V) -> Self {
        let json_value = serde_json::to_value(value)
            .expect("ParamsBuilder: Failed to serialize value during insert_unwrap");
        self.inner.insert(key.to_string(), json_value);
        self
    }

    pub fn build(self) -> Params {
        Params { inner: self.inner }
    }
}

pub trait ParamsContainer {
    fn set_params(&mut self, params: Params);
    fn get_params(&self) -> &Params;
    fn get_params_mut(&mut self) -> &mut Params;
}
