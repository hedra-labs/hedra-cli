pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyMinimaxSpeech25HdPreview {
    #[serde(default)]
    pub input: InputMinimaxSpeech25HdPreview,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMinimaxSpeech25HdPreview {
    pub fn builder() -> SubmitBodyMinimaxSpeech25HdPreviewBuilder {
        <SubmitBodyMinimaxSpeech25HdPreviewBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxSpeech25HdPreviewBuilder {
    input: Option<InputMinimaxSpeech25HdPreview>,
    webhook: Option<String>,
}

impl SubmitBodyMinimaxSpeech25HdPreviewBuilder {
    pub fn input(mut self, value: InputMinimaxSpeech25HdPreview) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxSpeech25HdPreview`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxSpeech25HdPreviewBuilder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxSpeech25HdPreview, BuildError> {
        Ok(SubmitBodyMinimaxSpeech25HdPreview {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

