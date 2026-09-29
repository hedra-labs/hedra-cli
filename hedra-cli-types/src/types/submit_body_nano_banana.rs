pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyNanoBanana {
    pub input: InputNanoBanana,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyNanoBanana {
    pub fn builder() -> SubmitBodyNanoBananaBuilder {
        <SubmitBodyNanoBananaBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyNanoBananaBuilder {
    input: Option<InputNanoBanana>,
    webhook: Option<String>,
}

impl SubmitBodyNanoBananaBuilder {
    pub fn input(mut self, value: InputNanoBanana) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyNanoBanana`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyNanoBananaBuilder::input)
    pub fn build(self) -> Result<SubmitBodyNanoBanana, BuildError> {
        Ok(SubmitBodyNanoBanana {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

