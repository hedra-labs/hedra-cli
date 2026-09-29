pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyQwenImage21 {
    pub input: InputQwenImage21,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyQwenImage21 {
    pub fn builder() -> SubmitBodyQwenImage21Builder {
        <SubmitBodyQwenImage21Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyQwenImage21Builder {
    input: Option<InputQwenImage21>,
    webhook: Option<String>,
}

impl SubmitBodyQwenImage21Builder {
    pub fn input(mut self, value: InputQwenImage21) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyQwenImage21`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyQwenImage21Builder::input)
    pub fn build(self) -> Result<SubmitBodyQwenImage21, BuildError> {
        Ok(SubmitBodyQwenImage21 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

