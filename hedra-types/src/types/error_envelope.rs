pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    #[serde(default)]
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<i64>,
}

impl ErrorEnvelope {
    pub fn builder() -> ErrorEnvelopeBuilder {
        <ErrorEnvelopeBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ErrorEnvelopeBuilder {
    code: Option<ErrorCode>,
    message: Option<String>,
    retryable: Option<bool>,
    retry_after: Option<i64>,
}

impl ErrorEnvelopeBuilder {
    pub fn code(mut self, value: ErrorCode) -> Self {
        self.code = Some(value);
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn retryable(mut self, value: bool) -> Self {
        self.retryable = Some(value);
        self
    }

    pub fn retry_after(mut self, value: i64) -> Self {
        self.retry_after = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ErrorEnvelope`].
    /// This method will fail if any of the following fields are not set:
    /// - [`code`](ErrorEnvelopeBuilder::code)
    /// - [`message`](ErrorEnvelopeBuilder::message)
    pub fn build(self) -> Result<ErrorEnvelope, BuildError> {
        Ok(ErrorEnvelope {
            code: self.code.ok_or_else(|| BuildError::missing_field("code"))?,
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
            retryable: self.retryable,
            retry_after: self.retry_after,
        })
    }
}
