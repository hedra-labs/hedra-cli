pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyWan30 {
    pub input: InputWan30,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyWan30 {
    pub fn builder() -> SubmitBodyWan30Builder {
        <SubmitBodyWan30Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyWan30Builder {
    input: Option<InputWan30>,
    webhook: Option<String>,
}

impl SubmitBodyWan30Builder {
    pub fn input(mut self, value: InputWan30) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyWan30`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyWan30Builder::input)
    pub fn build(self) -> Result<SubmitBodyWan30, BuildError> {
        Ok(SubmitBodyWan30 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

