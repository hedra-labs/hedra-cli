pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKlingO3Reference {
    pub input: InputKlingO3Reference,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyKlingO3Reference {
    pub fn builder() -> SubmitBodyKlingO3ReferenceBuilder {
        <SubmitBodyKlingO3ReferenceBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKlingO3ReferenceBuilder {
    input: Option<InputKlingO3Reference>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyKlingO3ReferenceBuilder {
    pub fn input(mut self, value: InputKlingO3Reference) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyKlingO3Reference`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKlingO3ReferenceBuilder::input)
    pub fn build(self) -> Result<SubmitBodyKlingO3Reference, BuildError> {
        Ok(SubmitBodyKlingO3Reference {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

