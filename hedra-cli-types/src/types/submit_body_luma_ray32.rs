pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitBodyLumaRay32 {
    pub input: InputLumaRay32,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyLumaRay32 {
    pub fn builder() -> SubmitBodyLumaRay32Builder {
        <SubmitBodyLumaRay32Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyLumaRay32Builder {
    input: Option<InputLumaRay32>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyLumaRay32Builder {
    pub fn input(mut self, value: InputLumaRay32) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyLumaRay32`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyLumaRay32Builder::input)
    pub fn build(self) -> Result<SubmitBodyLumaRay32, BuildError> {
        Ok(SubmitBodyLumaRay32 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

