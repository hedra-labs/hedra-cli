pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyElevenlabsAudioIsolation {
    pub input: InputElevenlabsAudioIsolation,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyElevenlabsAudioIsolation {
    pub fn builder() -> SubmitBodyElevenlabsAudioIsolationBuilder {
        <SubmitBodyElevenlabsAudioIsolationBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsAudioIsolationBuilder {
    input: Option<InputElevenlabsAudioIsolation>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyElevenlabsAudioIsolationBuilder {
    pub fn input(mut self, value: InputElevenlabsAudioIsolation) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsAudioIsolation`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsAudioIsolationBuilder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsAudioIsolation, BuildError> {
        Ok(SubmitBodyElevenlabsAudioIsolation {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

