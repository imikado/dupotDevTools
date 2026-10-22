use std::fmt;

/// Error reported by domain logic, carrying a message meant for the user.
///
/// Like `anyhow::Error`, it deliberately does not implement
/// `std::error::Error`, so that any library error converts into it with `?`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError(String);

impl DomainError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<E: std::error::Error> From<E> for DomainError {
    fn from(error: E) -> Self {
        let mut message = error.to_string();
        let mut source = error.source();
        while let Some(cause) = source {
            message.push_str(": ");
            message.push_str(&cause.to_string());
            source = cause.source();
        }
        Self(message)
    }
}

pub type DomainResult<T> = Result<T, DomainError>;
