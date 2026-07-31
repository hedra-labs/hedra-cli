pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyMinimaxSpeech25TurboPreview {
    #[serde(default)]
    pub input: InputMinimaxSpeech25TurboPreview,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Replays the original ack for a retried submit instead of enqueueing a duplicate job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl SubmitBodyMinimaxSpeech25TurboPreview {
    pub fn builder() -> SubmitBodyMinimaxSpeech25TurboPreviewBuilder {
        <SubmitBodyMinimaxSpeech25TurboPreviewBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxSpeech25TurboPreviewBuilder {
    input: Option<InputMinimaxSpeech25TurboPreview>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
}

impl SubmitBodyMinimaxSpeech25TurboPreviewBuilder {
    pub fn input(mut self, value: InputMinimaxSpeech25TurboPreview) -> Self {
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

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxSpeech25TurboPreview`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxSpeech25TurboPreviewBuilder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxSpeech25TurboPreview, BuildError> {
        Ok(SubmitBodyMinimaxSpeech25TurboPreview {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
        })
    }
}

