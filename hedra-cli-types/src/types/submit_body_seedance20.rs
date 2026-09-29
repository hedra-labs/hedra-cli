pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodySeedance20 {
    pub input: InputSeedance20,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodySeedance20 {
    pub fn builder() -> SubmitBodySeedance20Builder {
        <SubmitBodySeedance20Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodySeedance20Builder {
    input: Option<InputSeedance20>,
    webhook: Option<String>,
}

impl SubmitBodySeedance20Builder {
    pub fn input(mut self, value: InputSeedance20) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodySeedance20`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodySeedance20Builder::input)
    pub fn build(self) -> Result<SubmitBodySeedance20, BuildError> {
        Ok(SubmitBodySeedance20 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

