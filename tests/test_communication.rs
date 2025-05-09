use pocketflow_rs::communication::{Params, SharedStore};
use serde_json::{Value as JsonValue, json};

#[test]
fn test_shared_store_operations() {
    let store = SharedStore::new();

    // Test inserting and getting a String (via JsonValue)
    let val_str = json!("value1");
    store.insert("key_str", val_str.clone()); // Store JsonValue

    let retrieved_json_str = store
        .get::<JsonValue>("key_str")
        .expect("Failed to get key_str as JsonValue");
    let s_val: String =
        serde_json::from_value(retrieved_json_str).expect("Failed to deserialize to String");
    assert_eq!(s_val, "value1");

    // Test inserting and getting an integer (via JsonValue)
    let val_num = json!(42i32);
    store.insert("key_num", val_num.clone()); // Store JsonValue

    let retrieved_json_num = store
        .get::<JsonValue>("key_num")
        .expect("Failed to get key_num as JsonValue");
    let i_num: i32 =
        serde_json::from_value(retrieved_json_num).expect("Failed to deserialize to i32");
    assert_eq!(i_num, 42);

    // Test getting a value with the wrong direct type
    // If "key_num" stores a JsonValue(Number(42)), trying to get it as String directly will fail.
    assert_eq!(store.get::<String>("key_num"), None);

    // Test getting the raw JsonValue
    assert_eq!(store.get::<JsonValue>("key_str"), Some(json!("value1")));
    assert_eq!(store.get::<JsonValue>("key_num"), Some(json!(42)));

    // Test inserting and getting a non-Json type (e.g., Vec<i32>)
    let my_vec = vec![1, 2, 3];
    store.insert("key_vec", my_vec.clone());
    let retrieved_vec = store.get::<Vec<i32>>("key_vec");
    assert_eq!(retrieved_vec, Some(vec![1, 2, 3]));

    // Test trying to get a non-Json type as JsonValue
    assert_eq!(store.get::<JsonValue>("key_vec"), None);
}

#[test]
fn test_params() {
    let mut params = Params::new();
    params.set("filename", "test.txt").unwrap();
    assert_eq!(
        params.get::<String>("filename"),
        Some("test.txt".to_string())
    );
    let mut child_params = Params::new();
    child_params.set("mode", "read").unwrap();
    let merged = params.merge(&child_params);
    assert_eq!(
        merged.get::<String>("filename"),
        Some("test.txt".to_string())
    );
    assert_eq!(merged.get::<String>("mode"), Some("read".to_string()));
}
