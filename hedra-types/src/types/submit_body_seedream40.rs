pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodySeedream40 {
    pub input: InputSeedream40,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodySeedream40 {
    pub fn builder() -> SubmitBodySeedream40Builder {
        <SubmitBodySeedream40Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodySeedream40Builder {
    input: Option<InputSeedream40>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodySeedream40Builder {
    pub fn input(mut self, value: InputSeedream40) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodySeedream40`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodySeedream40Builder::input)
    pub fn build(self) -> Result<SubmitBodySeedream40, BuildError> {
        Ok(SubmitBodySeedream40 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

