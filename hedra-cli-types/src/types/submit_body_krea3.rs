pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKrea3 {
    pub input: InputKrea3,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyKrea3 {
    pub fn builder() -> SubmitBodyKrea3Builder {
        <SubmitBodyKrea3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKrea3Builder {
    input: Option<InputKrea3>,
    webhook: Option<String>,
}

impl SubmitBodyKrea3Builder {
    pub fn input(mut self, value: InputKrea3) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyKrea3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKrea3Builder::input)
    pub fn build(self) -> Result<SubmitBodyKrea3, BuildError> {
        Ok(SubmitBodyKrea3 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

