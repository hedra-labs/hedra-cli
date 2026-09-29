pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyGeminiOmniFlash {
    pub input: InputGeminiOmniFlash,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyGeminiOmniFlash {
    pub fn builder() -> SubmitBodyGeminiOmniFlashBuilder {
        <SubmitBodyGeminiOmniFlashBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyGeminiOmniFlashBuilder {
    input: Option<InputGeminiOmniFlash>,
    webhook: Option<String>,
}

impl SubmitBodyGeminiOmniFlashBuilder {
    pub fn input(mut self, value: InputGeminiOmniFlash) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyGeminiOmniFlash`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyGeminiOmniFlashBuilder::input)
    pub fn build(self) -> Result<SubmitBodyGeminiOmniFlash, BuildError> {
        Ok(SubmitBodyGeminiOmniFlash {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

