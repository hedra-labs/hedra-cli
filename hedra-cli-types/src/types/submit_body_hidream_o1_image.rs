pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyHidreamO1Image {
    pub input: InputHidreamO1Image,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyHidreamO1Image {
    pub fn builder() -> SubmitBodyHidreamO1ImageBuilder {
        <SubmitBodyHidreamO1ImageBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyHidreamO1ImageBuilder {
    input: Option<InputHidreamO1Image>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyHidreamO1ImageBuilder {
    pub fn input(mut self, value: InputHidreamO1Image) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyHidreamO1Image`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyHidreamO1ImageBuilder::input)
    pub fn build(self) -> Result<SubmitBodyHidreamO1Image, BuildError> {
        Ok(SubmitBodyHidreamO1Image {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

