pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyNanoBanana2 {
    pub input: InputNanoBanana2,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyNanoBanana2 {
    pub fn builder() -> SubmitBodyNanoBanana2Builder {
        <SubmitBodyNanoBanana2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyNanoBanana2Builder {
    input: Option<InputNanoBanana2>,
    webhook: Option<String>,
}

impl SubmitBodyNanoBanana2Builder {
    pub fn input(mut self, value: InputNanoBanana2) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyNanoBanana2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyNanoBanana2Builder::input)
    pub fn build(self) -> Result<SubmitBodyNanoBanana2, BuildError> {
        Ok(SubmitBodyNanoBanana2 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

