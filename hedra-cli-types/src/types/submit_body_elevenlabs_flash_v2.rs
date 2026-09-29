pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyElevenlabsFlashV2 {
    #[serde(default)]
    pub input: InputElevenlabsFlashV2,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyElevenlabsFlashV2 {
    pub fn builder() -> SubmitBodyElevenlabsFlashV2Builder {
        <SubmitBodyElevenlabsFlashV2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsFlashV2Builder {
    input: Option<InputElevenlabsFlashV2>,
    webhook: Option<String>,
}

impl SubmitBodyElevenlabsFlashV2Builder {
    pub fn input(mut self, value: InputElevenlabsFlashV2) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsFlashV2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsFlashV2Builder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsFlashV2, BuildError> {
        Ok(SubmitBodyElevenlabsFlashV2 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

