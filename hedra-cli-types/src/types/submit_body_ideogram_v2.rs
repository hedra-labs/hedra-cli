pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitBodyIdeogramV2 {
    pub input: InputIdeogramV2,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyIdeogramV2 {
    pub fn builder() -> SubmitBodyIdeogramV2Builder {
        <SubmitBodyIdeogramV2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyIdeogramV2Builder {
    input: Option<InputIdeogramV2>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyIdeogramV2Builder {
    pub fn input(mut self, value: InputIdeogramV2) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyIdeogramV2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyIdeogramV2Builder::input)
    pub fn build(self) -> Result<SubmitBodyIdeogramV2, BuildError> {
        Ok(SubmitBodyIdeogramV2 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

