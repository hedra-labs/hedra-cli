pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyVeedVideoBackgroundRemoval {
    pub input: InputVeedVideoBackgroundRemoval,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyVeedVideoBackgroundRemoval {
    pub fn builder() -> SubmitBodyVeedVideoBackgroundRemovalBuilder {
        <SubmitBodyVeedVideoBackgroundRemovalBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyVeedVideoBackgroundRemovalBuilder {
    input: Option<InputVeedVideoBackgroundRemoval>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyVeedVideoBackgroundRemovalBuilder {
    pub fn input(mut self, value: InputVeedVideoBackgroundRemoval) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyVeedVideoBackgroundRemoval`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyVeedVideoBackgroundRemovalBuilder::input)
    pub fn build(self) -> Result<SubmitBodyVeedVideoBackgroundRemoval, BuildError> {
        Ok(SubmitBodyVeedVideoBackgroundRemoval {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

