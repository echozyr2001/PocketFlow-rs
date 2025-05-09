use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// SharedStore provides global data sharing between nodes
/// Similar to heap memory - shared by all nodes
#[derive(Clone, Default)]
pub struct SharedStore {
    inner: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl SharedStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a value of any type that is 'static, Send, and Sync into the store.
    pub fn insert<T: 'static + Send + Sync>(&self, key: &str, value: T) {
        self.inner.write().insert(key.to_string(), Box::new(value));
    }

    /// Get a clone of a value from the store, downcasting it to type T.
    /// Returns None if the key does not exist or if the stored value is not of type T.
    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        self.inner
            .read()
            .get(key)
            .and_then(|b| b.downcast_ref::<T>())
            .cloned()
    }

    /// Remove a value from the store.
    /// Returns the boxed value if the key existed.
    pub fn remove(&self, key: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.inner.write().remove(key)
    }

    /// Check if a key exists.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.read().contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.inner.read().keys().cloned().collect()
    }
}

/// Params represents immutable configuration for nodes
/// Similar to stack memory - passed down from parent to child
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

    /// Create params from a HashMap
    pub fn from_map(map: HashMap<String, JsonValue>) -> Self {
        Self { inner: map }
    }

    /// Set a parameter value
    pub fn set<V: Serialize>(&mut self, key: &str, value: V) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.inner.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Get a parameter value and deserialize to type T
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inner
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a reference to the raw JsonValue
    pub fn get_value(&self, key: &str) -> Option<&JsonValue> {
        self.inner.get(key)
    }

    /// Creates a new ParamsBuilder instance.
    pub fn builder() -> ParamsBuilder {
        ParamsBuilder::new()
    }

    /// Merge with another Params instance
    /// Used when combining parent and child params
    pub fn merge(&self, other: &Params) -> Self {
        let mut new_map = self.inner.clone();
        new_map.extend(other.inner.clone());
        Self { inner: new_map }
    }

    /// Get all keys
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

/// ParamsContainer provides params management for nodes and flows
pub trait ParamsContainer {
    /// Set params for this container
    fn set_params(&mut self, params: Params);

    /// Get current params
    fn get_params(&self) -> &Params;

    /// Get mutable reference to params
    fn get_params_mut(&mut self) -> &mut Params;
}
