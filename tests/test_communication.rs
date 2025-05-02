use pocketflow_rs::communication::{Params, SharedStore};
use serde_json::Value as JsonValue;

#[test]
fn test_shared_store_json() {
    let store = SharedStore::new();
    store.insert_json("key1", "value1");
    assert_eq!(store.get_json::<String>("key1"), Some("value1".to_string()));
    store.insert_json("num", 42i32);
    assert_eq!(store.get_json::<i32>("num"), Some(42));
    assert_eq!(store.get_json::<String>("num"), None); // Wrong type returns None
    assert_eq!(
        store.get_json_value("key1"),
        Some(JsonValue::String("value1".to_string()))
    );
}

#[test]
fn test_shared_store_any() {
    let store = SharedStore::new();
    store.insert_any("vec", vec![1, 2, 3]);
    let v = store.get_any::<Vec<i32>>("vec");
    assert_eq!(v, Some(vec![1, 2, 3]));
}

#[test]
fn test_params() {
    let mut params = Params::new();
    params.set("filename", "test.txt");
    assert_eq!(
        params.get::<String>("filename"),
        Some("test.txt".to_string())
    );
    let mut child_params = Params::new();
    child_params.set("mode", "read");
    let merged = params.merge(&child_params);
    assert_eq!(
        merged.get::<String>("filename"),
        Some("test.txt".to_string())
    );
    assert_eq!(merged.get::<String>("mode"), Some("read".to_string()));
}
