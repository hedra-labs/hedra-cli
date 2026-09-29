pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyElevenlabsFlashMultilingualV2 {
    #[serde(default)]
    pub input: InputElevenlabsFlashMultilingualV2,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyElevenlabsFlashMultilingualV2 {
    pub fn builder() -> SubmitBodyElevenlabsFlashMultilingualV2Builder {
        <SubmitBodyElevenlabsFlashMultilingualV2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsFlashMultilingualV2Builder {
    input: Option<InputElevenlabsFlashMultilingualV2>,
    webhook: Option<String>,
}

impl SubmitBodyElevenlabsFlashMultilingualV2Builder {
    pub fn input(mut self, value: InputElevenlabsFlashMultilingualV2) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsFlashMultilingualV2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsFlashMultilingualV2Builder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsFlashMultilingualV2, BuildError> {
        Ok(SubmitBodyElevenlabsFlashMultilingualV2 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

