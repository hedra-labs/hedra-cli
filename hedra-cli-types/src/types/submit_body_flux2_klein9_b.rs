pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyFlux2Klein9B {
    pub input: InputFlux2Klein9B,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyFlux2Klein9B {
    pub fn builder() -> SubmitBodyFlux2Klein9BBuilder {
        <SubmitBodyFlux2Klein9BBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFlux2Klein9BBuilder {
    input: Option<InputFlux2Klein9B>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyFlux2Klein9BBuilder {
    pub fn input(mut self, value: InputFlux2Klein9B) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyFlux2Klein9B`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFlux2Klein9BBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFlux2Klein9B, BuildError> {
        Ok(SubmitBodyFlux2Klein9B {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

