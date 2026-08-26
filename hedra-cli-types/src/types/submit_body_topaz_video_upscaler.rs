pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyTopazVideoUpscaler {
    pub input: InputTopazVideoUpscaler,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyTopazVideoUpscaler {
    pub fn builder() -> SubmitBodyTopazVideoUpscalerBuilder {
        <SubmitBodyTopazVideoUpscalerBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyTopazVideoUpscalerBuilder {
    input: Option<InputTopazVideoUpscaler>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyTopazVideoUpscalerBuilder {
    pub fn input(mut self, value: InputTopazVideoUpscaler) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyTopazVideoUpscaler`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyTopazVideoUpscalerBuilder::input)
    pub fn build(self) -> Result<SubmitBodyTopazVideoUpscaler, BuildError> {
        Ok(SubmitBodyTopazVideoUpscaler {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

