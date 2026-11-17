pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyGeminiOmniFlash12 {
    pub input: InputGeminiOmniFlash12,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyGeminiOmniFlash12 {
    pub fn builder() -> SubmitBodyGeminiOmniFlash12Builder {
        <SubmitBodyGeminiOmniFlash12Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyGeminiOmniFlash12Builder {
    input: Option<InputGeminiOmniFlash12>,
    webhook: Option<String>,
}

impl SubmitBodyGeminiOmniFlash12Builder {
    pub fn input(mut self, value: InputGeminiOmniFlash12) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyGeminiOmniFlash12`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyGeminiOmniFlash12Builder::input)
    pub fn build(self) -> Result<SubmitBodyGeminiOmniFlash12, BuildError> {
        Ok(SubmitBodyGeminiOmniFlash12 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

