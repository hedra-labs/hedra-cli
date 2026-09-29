pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyGrokVideo {
    pub input: InputGrokVideo,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyGrokVideo {
    pub fn builder() -> SubmitBodyGrokVideoBuilder {
        <SubmitBodyGrokVideoBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyGrokVideoBuilder {
    input: Option<InputGrokVideo>,
    webhook: Option<String>,
}

impl SubmitBodyGrokVideoBuilder {
    pub fn input(mut self, value: InputGrokVideo) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyGrokVideo`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyGrokVideoBuilder::input)
    pub fn build(self) -> Result<SubmitBodyGrokVideo, BuildError> {
        Ok(SubmitBodyGrokVideo {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

