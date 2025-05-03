use std::cell::RefCell;

use pocketflow_rs::{
    communication::SharedStore,
    core::{Action, Result},
    node::{Node, RetryableNode},
};

struct SimpleNode;

impl Node for SimpleNode {
    type PrepResult = String;
    type ExecResult = usize;

    fn prep(&self, shared: &SharedStore) -> Result<Self::PrepResult> {
        Ok(shared.get_json::<String>("input").unwrap_or_default())
    }

    fn exec(&self, prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
        Ok(prep_res.len())
    }

    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: Self::PrepResult,
        exec_res: Self::ExecResult,
    ) -> Result<Action> {
        Ok(format!("len={exec_res}").as_str().into())
    }
}

#[test]
fn test_run_node() {
    let node = SimpleNode;
    let store = SharedStore::new();
    store.insert_json("input", "hello");

    let result = node.run(&store).unwrap();
    assert_eq!(result, "len=5".into());
}

// ------------------------------------
// 2. Retryable Node: fails once, then works
// ------------------------------------
struct RetryOnceNode {
    attempts: RefCell<usize>,
}

impl Node for RetryOnceNode {
    type PrepResult = ();
    type ExecResult = usize;

    fn exec(&self, _prep: Self::PrepResult) -> Result<Self::ExecResult> {
        let mut guard = self.attempts.borrow_mut();
        if *guard == 0 {
            *guard += 1;
            Err(anyhow::anyhow!("fail once"))
        } else {
            Ok(*guard)
        }
    }

    fn post(
        &self,
        _shared: &SharedStore,
        _prep: Self::PrepResult,
        exec: Self::ExecResult,
    ) -> Result<Action> {
        Ok(format!("attempts={exec}").into())
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
        attempts: RefCell::new(0),
    };
    let store = SharedStore::new();

    let result = node.run_with_retry(&store).unwrap();
    assert_eq!(result, "attempts=1".into());
}

// ------------------------------------
// 3. Retryable Node: always fails
// ------------------------------------
struct AlwaysFailNode;

impl Node for AlwaysFailNode {
    type PrepResult = ();
    type ExecResult = ();

    fn exec(&self, _prep: Self::PrepResult) -> Result<Self::ExecResult> {
        Err(anyhow::anyhow!("boom"))
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

    let result = node.run_with_retry(&store);
    assert!(result.is_err());
}
