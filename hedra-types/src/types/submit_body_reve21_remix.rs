pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyReve21Remix {
    pub input: InputReve21Remix,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyReve21Remix {
    pub fn builder() -> SubmitBodyReve21RemixBuilder {
        <SubmitBodyReve21RemixBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyReve21RemixBuilder {
    input: Option<InputReve21Remix>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyReve21RemixBuilder {
    pub fn input(mut self, value: InputReve21Remix) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyReve21Remix`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyReve21RemixBuilder::input)
    pub fn build(self) -> Result<SubmitBodyReve21Remix, BuildError> {
        Ok(SubmitBodyReve21Remix {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

