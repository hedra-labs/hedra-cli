pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyElevenlabsV3 {
    #[serde(default)]
    pub input: InputElevenlabsV3,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyElevenlabsV3 {
    pub fn builder() -> SubmitBodyElevenlabsV3Builder {
        <SubmitBodyElevenlabsV3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsV3Builder {
    input: Option<InputElevenlabsV3>,
    webhook: Option<String>,
}

impl SubmitBodyElevenlabsV3Builder {
    pub fn input(mut self, value: InputElevenlabsV3) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsV3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsV3Builder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsV3, BuildError> {
        Ok(SubmitBodyElevenlabsV3 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

