use parking_lot::Mutex;
use pocketflow_rs::{
    communication::SharedStore,
    core::{Action, DEFAULT_ACTION, ExecResult, PrepResult, Result},
    node::{BaseNode, RetryableNode},
};

// ------------------------------------
// 1. Default Node: get default action
// ------------------------------------
struct DefaultNode;

impl BaseNode for DefaultNode {}

#[test]
fn test_default_node() {
    let node = DefaultNode;
    let store = SharedStore::new();
    store.insert_json("input", "hello");

    let result = node.run(&store);
    match result {
        Ok(action) => assert_eq!(action, DEFAULT_ACTION),
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 2. SimpleNode Node: simply run a function
// ------------------------------------
struct SimpleNode;

impl BaseNode for SimpleNode {
    fn prep(&self, shared: &SharedStore) -> Result<PrepResult> {
        let input = shared.get_json::<String>("input").unwrap_or_default();
        Ok(serde_json::Value::String(input).into())
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        if let Some(s) = prep_res.as_str() {
            let length = s.len();
            Ok(serde_json::Value::Number(length.into()).into())
        } else {
            Err(anyhow::anyhow!("Expected string in prep_res"))
        }
    }

    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<Action> {
        if let Some(n) = exec_res.as_u64() {
            Ok(format!("len={}", n).into())
        } else {
            Err(anyhow::anyhow!("Expected number in exec_res"))
        }
    }
}

#[test]
fn test_run_node() {
    let node = SimpleNode;
    let store = SharedStore::new();
    store.insert_json("input", "hello");

    let result = node.run(&store);
    match result {
        Ok(action) => assert_eq!(action, "len=5".into()),
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 3. Retryable Node: fails once, then works
// ------------------------------------
struct RetryOnceNode {
    attempts: Mutex<usize>,
}

impl BaseNode for RetryOnceNode {
    fn exec(&self, _prep: &PrepResult) -> Result<ExecResult> {
        let mut guard = self.attempts.lock();
        if *guard == 0 {
            *guard += 1;
            Err(anyhow::anyhow!("fail once"))
        } else {
            // Convert attempts count to ExecResult
            Ok(serde_json::Value::Number((*guard).into()).into())
        }
    }

    fn post(
        &self,
        _shared: &SharedStore,
        _prep: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<Action> {
        if let Some(n) = exec_res.as_u64() {
            Ok(format!("attempts={}", n).into())
        } else {
            Err(anyhow::anyhow!("Expected number in exec_res"))
        }
    }

    fn run(&self, shared: &SharedStore) -> Result<Action> {
        self.run_with_retry(shared)
    }
}

impl RetryableNode for RetryOnceNode {
    fn get_max_retries(&self) -> u32 {
        3
    }
    fn get_wait_ms(&self) -> u64 {
        5
    }
}

#[test]
fn test_retryable_node_retries_and_succeeds() {
    let node = RetryOnceNode {
        attempts: Mutex::new(0),
    };
    let store = SharedStore::new();

    let result = node.run(&store);
    match result {
        Ok(action) => assert_eq!(action, "attempts=1".into()),
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 4. Retryable Node: always fails
// ------------------------------------
struct AlwaysFailNode;

impl BaseNode for AlwaysFailNode {
    fn exec(&self, _prep: &PrepResult) -> Result<ExecResult> {
        Err(anyhow::anyhow!("boom"))
    }

    fn run(&self, shared: &SharedStore) -> Result<Action> {
        self.run_with_retry(shared)
    }
}

impl RetryableNode for AlwaysFailNode {
    fn get_max_retries(&self) -> u32 {
        2
    }
    fn get_wait_ms(&self) -> u64 {
        5
    }
}

#[test]
fn test_retryable_node_fails_all_attempts() {
    let node = AlwaysFailNode;
    let store = SharedStore::new();

    let result = node.run(&store);
    assert!(result.is_err());
}
