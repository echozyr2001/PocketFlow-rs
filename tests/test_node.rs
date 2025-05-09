use anyhow::Result;
use async_trait::async_trait;
use parking_lot::Mutex;
use pocketflow_rs::{
    communication::{BaseSharedStore, SharedStore},
    core::{
        ExecResult, PostResult, PrepResult,
        node::{NodeTrait, RetryConfig},
    },
};
use serde_json::{Value as JsonValue, json};

// Define a default PostResult for comparison, if needed for tests
const DEFAULT_POST_RESULT_VAL: &str = "default_node_action";

// ------------------------------------
// 1. Default Node: get default action
// ------------------------------------
struct DefaultNode;

#[async_trait]
impl NodeTrait for DefaultNode {
    // Override post to return a specific default if needed, otherwise NodeTrait's default is PostResult("")
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from(DEFAULT_POST_RESULT_VAL))
    }
}

#[test]
fn test_default_node() {
    let node = DefaultNode;
    let store = BaseSharedStore::new_in_memory(); // Use BaseSharedStore for instantiation
    // store.insert("input", json!("hello")); // BaseSharedStore struct has generic insert
    // For &dyn SharedStore, we'd use:
    // store.insert_value("input", Arc::new(json!("hello")));
    // Since 'store' is a BaseSharedStore, we can use its convenient generic insert.
    store.insert("input", json!("hello"));

    let result = node.run(&store); // Pass &store which impls &dyn SharedStore
    match result {
        Ok(post_res) => assert_eq!(post_res, PostResult::from(DEFAULT_POST_RESULT_VAL)),
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 2. SimpleNode Node: simply run a function
// ------------------------------------
struct SimpleNode;

#[async_trait]
impl NodeTrait for SimpleNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        let input_json_val: Option<JsonValue> = shared_store
            .get_value("input") // This is from SharedStore trait, returns Option<StoredValue>
            .and_then(|arc_any| arc_any.downcast_ref::<JsonValue>().cloned());

        let input_str = input_json_val
            .unwrap_or(JsonValue::Null)
            .as_str()
            .unwrap_or_default()
            .to_string();
        Ok(PrepResult::from(JsonValue::String(input_str)))
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        if let Some(s) = prep_res.as_str() {
            let length = s.len();
            Ok(ExecResult::from(JsonValue::Number(length.into())))
        } else {
            Err(anyhow::anyhow!("Expected string in prep_res"))
        }
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        if let Some(n) = exec_res.as_u64() {
            Ok(PostResult::from(format!("len={}", n)))
        } else {
            Err(anyhow::anyhow!("Expected number in exec_res"))
        }
    }
}

#[test]
fn test_run_node() {
    let node = SimpleNode;
    let store = BaseSharedStore::new_in_memory();
    store.insert("input", json!("hello")); // Uses BaseSharedStore's generic insert

    let result = node.run(&store); // Pass &store which impls &dyn SharedStore
    match result {
        Ok(post_res) => assert_eq!(post_res, PostResult::from("len=5")),
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 3. Retryable Node: fails once, then works
// ------------------------------------
struct RetryOnceNode {
    attempts: Mutex<usize>,
    retry_config: Option<RetryConfig>,
}

#[async_trait]
impl NodeTrait for RetryOnceNode {
    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        let execution_logic = || {
            let mut guard = self.attempts.lock();
            if *guard == 0 {
                *guard += 1;
                println!("RetryOnceNode: Exec attempt failing (attempt {})", *guard);
                Err(anyhow::anyhow!("fail once"))
            } else {
                println!(
                    "RetryOnceNode: Exec attempt succeeding (attempt {})",
                    *guard
                );
                Ok(ExecResult::from(JsonValue::Number((*guard).into())))
            }
        };

        if let Some(config) = &self.retry_config {
            config.exec_with_retry(execution_logic)
        } else {
            execution_logic()
        }
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        if let Some(n) = exec_res.as_u64() {
            Ok(PostResult::from(format!("attempts={}", n)))
        } else {
            Err(anyhow::anyhow!("Expected number in exec_res"))
        }
    }
    // run will be inherited from NodeTrait default
}

#[test]
fn test_retryable_node_retries_and_succeeds() {
    let node = RetryOnceNode {
        attempts: Mutex::new(0),
        retry_config: Some(RetryConfig::new(3, 0.005)), // 3 total attempts, 5ms wait
    };
    let store = BaseSharedStore::new_in_memory();

    let result = node.run(&store);
    match result {
        Ok(post_res) => assert_eq!(post_res, PostResult::from("attempts=1")), // Should succeed on 2nd attempt (attempts becomes 1)
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 4. Retryable Node: always fails
// ------------------------------------
struct AlwaysFailNode {
    retry_config: Option<RetryConfig>,
}

#[async_trait]
impl NodeTrait for AlwaysFailNode {
    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        let execution_logic = || -> Result<ExecResult> {
            println!("AlwaysFailNode: Exec attempt failing");
            Err(anyhow::anyhow!("boom"))
        };

        if let Some(config) = &self.retry_config {
            config.exec_with_retry(execution_logic)
        } else {
            execution_logic()
        }
    }
    // run will be inherited
}

#[test]
fn test_retryable_node_fails_all_attempts() {
    let node = AlwaysFailNode {
        retry_config: Some(RetryConfig::new(2, 0.005)), // 2 total attempts
    };
    let store = BaseSharedStore::new_in_memory();

    let result = node.run(&store);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "boom");
    }
}
