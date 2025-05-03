#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action<'a>(pub Option<&'a str>);

pub static DEFAULT_ACTION: Action = Action(Some("default"));
pub static NONE_ACTION: Action = Action(None);

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
