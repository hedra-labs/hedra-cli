pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyElevenlabsMultilingualV2 {
    #[serde(default)]
    pub input: InputElevenlabsMultilingualV2,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyElevenlabsMultilingualV2 {
    pub fn builder() -> SubmitBodyElevenlabsMultilingualV2Builder {
        <SubmitBodyElevenlabsMultilingualV2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsMultilingualV2Builder {
    input: Option<InputElevenlabsMultilingualV2>,
    webhook: Option<String>,
}

impl SubmitBodyElevenlabsMultilingualV2Builder {
    pub fn input(mut self, value: InputElevenlabsMultilingualV2) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsMultilingualV2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsMultilingualV2Builder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsMultilingualV2, BuildError> {
        Ok(SubmitBodyElevenlabsMultilingualV2 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

