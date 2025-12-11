use std::fmt::Display;

/// Unique identifier for a captcha solving task
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(String);

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TaskId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TaskId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for TaskId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}