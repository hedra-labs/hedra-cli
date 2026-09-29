pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKrea2 {
    pub input: InputKrea2,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyKrea2 {
    pub fn builder() -> SubmitBodyKrea2Builder {
        <SubmitBodyKrea2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKrea2Builder {
    input: Option<InputKrea2>,
    webhook: Option<String>,
}

impl SubmitBodyKrea2Builder {
    pub fn input(mut self, value: InputKrea2) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyKrea2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKrea2Builder::input)
    pub fn build(self) -> Result<SubmitBodyKrea2, BuildError> {
        Ok(SubmitBodyKrea2 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

