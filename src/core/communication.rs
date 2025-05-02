use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// StoreValue: Support serde_json::Value and any types
#[derive(Debug)]
pub enum StoreValue {
    Any(Box<dyn Any + Send + Sync>),
    Json(JsonValue),
    // Can be extended to other types
}

impl StoreValue {
    pub fn as_json(&self) -> Option<&JsonValue> {
        match self {
            StoreValue::Json(j) => Some(j),
            _ => None,
        }
    }
    pub fn as_any<T: 'static>(&self) -> Option<&T> {
        match self {
            StoreValue::Any(b) => b.downcast_ref::<T>(),
            _ => None,
        }
    }
    pub fn into_json(self) -> Option<JsonValue> {
        match self {
            StoreValue::Json(j) => Some(j),
            _ => None,
        }
    }
    pub fn into_any<T: 'static>(self) -> Option<Box<T>> {
        match self {
            StoreValue::Any(b) => b.downcast::<T>().ok(),
            _ => None,
        }
    }
}

/// SharedStore provides global data sharing between nodes
/// Similar to heap memory - shared by all nodes
#[derive(Clone, Default)]
pub struct SharedStore {
    inner: Arc<RwLock<HashMap<String, StoreValue>>>,
}

impl SharedStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a serde_json::Value into the store
    pub fn insert_json<V: Serialize>(&self, key: &str, value: V) {
        let json_value = serde_json::to_value(value).unwrap();
        self.inner
            .write()
            .insert(key.to_string(), StoreValue::Json(json_value));
    }

    /// Insert any type into the store (advanced use)
    pub fn insert_any<T: 'static + Send + Sync>(&self, key: &str, value: T) {
        self.inner
            .write()
            .insert(key.to_string(), StoreValue::Any(Box::new(value)));
    }

    /// Get a value from the store and deserialize to type T (if Json)
    pub fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inner
            .read()
            .get(key)
            .and_then(|v| v.as_json())
            .and_then(|j| serde_json::from_value(j.clone()).ok())
    }

    /// Get a reference to the raw JsonValue
    pub fn get_json_value(&self, key: &str) -> Option<JsonValue> {
        self.inner
            .read()
            .get(key)
            .and_then(|v| v.as_json().cloned())
    }

    /// Get a reference to an Any value (advanced use)
    pub fn get_any<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        self.inner
            .read()
            .get(key)
            .and_then(|v| v.as_any::<T>())
            .cloned()
    }

    /// Update a value in the store (Json only)
    pub fn update_json<V: Serialize>(&self, key: &str, value: V) -> Option<()> {
        let json_value = serde_json::to_value(value).ok()?;
        let mut guard = self.inner.write();
        if let Some(StoreValue::Json(_)) = guard.get(key) {
            guard.insert(key.to_string(), StoreValue::Json(json_value));
            Some(())
        } else {
            None
        }
    }

    /// Remove a value
    pub fn remove(&self, key: &str) -> Option<StoreValue> {
        self.inner.write().remove(key)
    }

    /// Check if a key exists
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
    pub fn set<V: Serialize>(&mut self, key: &str, value: V) {
        let json_value = serde_json::to_value(value).unwrap();
        self.inner.insert(key.to_string(), json_value);
    }

    /// Get a parameter value and deserialize to type T
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inner
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a reference to the raw JsonValue
    pub fn get_json(&self, key: &str) -> Option<&JsonValue> {
        self.inner.get(key)
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

/// ParamsContainer provides params management for nodes and flows
pub trait ParamsContainer {
    /// Set params for this container
    fn set_params(&mut self, params: Params);

    /// Get current params
    fn get_params(&self) -> &Params;

    /// Get mutable reference to params
    fn get_params_mut(&mut self) -> &mut Params;
}
