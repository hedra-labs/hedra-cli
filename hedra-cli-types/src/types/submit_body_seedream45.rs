pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodySeedream45 {
    pub input: InputSeedream45,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodySeedream45 {
    pub fn builder() -> SubmitBodySeedream45Builder {
        <SubmitBodySeedream45Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodySeedream45Builder {
    input: Option<InputSeedream45>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodySeedream45Builder {
    pub fn input(mut self, value: InputSeedream45) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodySeedream45`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodySeedream45Builder::input)
    pub fn build(self) -> Result<SubmitBodySeedream45, BuildError> {
        Ok(SubmitBodySeedream45 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

