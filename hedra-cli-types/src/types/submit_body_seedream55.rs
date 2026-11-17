pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodySeedream55 {
    pub input: InputSeedream55,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodySeedream55 {
    pub fn builder() -> SubmitBodySeedream55Builder {
        <SubmitBodySeedream55Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodySeedream55Builder {
    input: Option<InputSeedream55>,
    webhook: Option<String>,
}

impl SubmitBodySeedream55Builder {
    pub fn input(mut self, value: InputSeedream55) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodySeedream55`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodySeedream55Builder::input)
    pub fn build(self) -> Result<SubmitBodySeedream55, BuildError> {
        Ok(SubmitBodySeedream55 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

