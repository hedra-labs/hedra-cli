pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKling26Pro {
    pub input: InputKling26Pro,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyKling26Pro {
    pub fn builder() -> SubmitBodyKling26ProBuilder {
        <SubmitBodyKling26ProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKling26ProBuilder {
    input: Option<InputKling26Pro>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyKling26ProBuilder {
    pub fn input(mut self, value: InputKling26Pro) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyKling26Pro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKling26ProBuilder::input)
    pub fn build(self) -> Result<SubmitBodyKling26Pro, BuildError> {
        Ok(SubmitBodyKling26Pro {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

