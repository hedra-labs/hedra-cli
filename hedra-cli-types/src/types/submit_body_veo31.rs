pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyVeo31 {
    pub input: InputVeo31,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyVeo31 {
    pub fn builder() -> SubmitBodyVeo31Builder {
        <SubmitBodyVeo31Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyVeo31Builder {
    input: Option<InputVeo31>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyVeo31Builder {
    pub fn input(mut self, value: InputVeo31) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyVeo31`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyVeo31Builder::input)
    pub fn build(self) -> Result<SubmitBodyVeo31, BuildError> {
        Ok(SubmitBodyVeo31 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

