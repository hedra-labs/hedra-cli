pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodySeedream50Lite {
    pub input: InputSeedream50Lite,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodySeedream50Lite {
    pub fn builder() -> SubmitBodySeedream50LiteBuilder {
        <SubmitBodySeedream50LiteBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodySeedream50LiteBuilder {
    input: Option<InputSeedream50Lite>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodySeedream50LiteBuilder {
    pub fn input(mut self, value: InputSeedream50Lite) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodySeedream50Lite`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodySeedream50LiteBuilder::input)
    pub fn build(self) -> Result<SubmitBodySeedream50Lite, BuildError> {
        Ok(SubmitBodySeedream50Lite {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

