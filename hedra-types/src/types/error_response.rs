pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Top-level error body: ``{"error": {...}}``.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorResponse {
    pub error: ErrorEnvelope,
}

impl ErrorResponse {
    pub fn builder() -> ErrorResponseBuilder {
        <ErrorResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ErrorResponseBuilder {
    error: Option<ErrorEnvelope>,
}

impl ErrorResponseBuilder {
    pub fn error(mut self, value: ErrorEnvelope) -> Self {
        self.error = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ErrorResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`error`](ErrorResponseBuilder::error)
    pub fn build(self) -> Result<ErrorResponse, BuildError> {
        Ok(ErrorResponse {
            error: self.error.ok_or_else(|| BuildError::missing_field("error"))?,
        })
    }
}
