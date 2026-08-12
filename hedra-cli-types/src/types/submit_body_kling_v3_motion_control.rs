pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKlingV3MotionControl {
    pub input: InputKlingV3MotionControl,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyKlingV3MotionControl {
    pub fn builder() -> SubmitBodyKlingV3MotionControlBuilder {
        <SubmitBodyKlingV3MotionControlBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKlingV3MotionControlBuilder {
    input: Option<InputKlingV3MotionControl>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyKlingV3MotionControlBuilder {
    pub fn input(mut self, value: InputKlingV3MotionControl) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyKlingV3MotionControl`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKlingV3MotionControlBuilder::input)
    pub fn build(self) -> Result<SubmitBodyKlingV3MotionControl, BuildError> {
        Ok(SubmitBodyKlingV3MotionControl {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

