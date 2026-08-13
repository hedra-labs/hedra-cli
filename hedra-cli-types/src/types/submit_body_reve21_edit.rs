pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyReve21Edit {
    pub input: InputReve21Edit,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyReve21Edit {
    pub fn builder() -> SubmitBodyReve21EditBuilder {
        <SubmitBodyReve21EditBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyReve21EditBuilder {
    input: Option<InputReve21Edit>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyReve21EditBuilder {
    pub fn input(mut self, value: InputReve21Edit) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyReve21Edit`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyReve21EditBuilder::input)
    pub fn build(self) -> Result<SubmitBodyReve21Edit, BuildError> {
        Ok(SubmitBodyReve21Edit {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

