pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyOmnihuman20 {
    pub input: InputOmnihuman20,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyOmnihuman20 {
    pub fn builder() -> SubmitBodyOmnihuman20Builder {
        <SubmitBodyOmnihuman20Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyOmnihuman20Builder {
    input: Option<InputOmnihuman20>,
    webhook: Option<String>,
}

impl SubmitBodyOmnihuman20Builder {
    pub fn input(mut self, value: InputOmnihuman20) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyOmnihuman20`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyOmnihuman20Builder::input)
    pub fn build(self) -> Result<SubmitBodyOmnihuman20, BuildError> {
        Ok(SubmitBodyOmnihuman20 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

