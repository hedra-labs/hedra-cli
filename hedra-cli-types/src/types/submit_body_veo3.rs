pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyVeo3 {
    pub input: InputVeo3,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyVeo3 {
    pub fn builder() -> SubmitBodyVeo3Builder {
        <SubmitBodyVeo3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyVeo3Builder {
    input: Option<InputVeo3>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyVeo3Builder {
    pub fn input(mut self, value: InputVeo3) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    pub fn idempotency_key(mut self, value: impl Into<String>) -> Self {
        self.idempotency_key = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyVeo3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyVeo3Builder::input)
    pub fn build(self) -> Result<SubmitBodyVeo3, BuildError> {
        Ok(SubmitBodyVeo3 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

