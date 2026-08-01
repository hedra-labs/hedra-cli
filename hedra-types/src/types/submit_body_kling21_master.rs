pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKling21Master {
    pub input: InputKling21Master,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyKling21Master {
    pub fn builder() -> SubmitBodyKling21MasterBuilder {
        <SubmitBodyKling21MasterBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKling21MasterBuilder {
    input: Option<InputKling21Master>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyKling21MasterBuilder {
    pub fn input(mut self, value: InputKling21Master) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyKling21Master`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKling21MasterBuilder::input)
    pub fn build(self) -> Result<SubmitBodyKling21Master, BuildError> {
        Ok(SubmitBodyKling21Master {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

