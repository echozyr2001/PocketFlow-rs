#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action(pub Option<String>);

impl From<String> for Action {
    fn from(value: String) -> Self {
        Action(Some(value))
    }
}

impl From<&str> for Action {
    fn from(value: &str) -> Self {
        Action(Some(value.to_string()))
    }
}
