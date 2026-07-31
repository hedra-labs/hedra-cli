pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitBodyImagen4 {
    pub input: InputImagen4,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyImagen4 {
    pub fn builder() -> SubmitBodyImagen4Builder {
        <SubmitBodyImagen4Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyImagen4Builder {
    input: Option<InputImagen4>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyImagen4Builder {
    pub fn input(mut self, value: InputImagen4) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyImagen4`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyImagen4Builder::input)
    pub fn build(self) -> Result<SubmitBodyImagen4, BuildError> {
        Ok(SubmitBodyImagen4 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

