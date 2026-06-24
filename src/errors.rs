use std::fmt;

pub type ShimResult<T> = Result<T, ShimError>;

#[derive(Debug, Clone)]
pub struct ShimError(pub String);

impl ShimError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ShimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ShimError {}

impl From<std::io::Error> for ShimError {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}
