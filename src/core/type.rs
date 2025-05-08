use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action<'a>(pub Option<&'a str>);

pub static DEFAULT_ACTION: Action = Action(Some("default"));
pub static NONE_ACTION: Action = Action(None);

impl Default for Action<'_> {
    fn default() -> Self {
        DEFAULT_ACTION
    }
}

impl From<String> for Action<'_> {
    fn from(value: String) -> Self {
        Action(Some(Box::leak(value.into_boxed_str())))
    }
}

impl<'a> From<&'a str> for Action<'a> {
    fn from(value: &'a str) -> Self {
        Action(Some(value))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrepResult(Value);

impl From<Value> for PrepResult {
    fn from(value: Value) -> Self {
        PrepResult(value)
    }
}

impl PrepResult {
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_str()
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.0.as_u64()
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        self.0.as_array()
    }

    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        self.0.as_object()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecResult(Value);

impl From<Value> for ExecResult {
    fn from(value: Value) -> Self {
        ExecResult(value)
    }
}

impl ExecResult {
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_str()
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.0.as_u64()
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        self.0.as_array()
    }

    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        self.0.as_object()
    }
}
