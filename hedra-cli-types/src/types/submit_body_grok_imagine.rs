pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyGrokImagine {
    #[serde(default)]
    pub input: InputGrokImagine,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyGrokImagine {
    pub fn builder() -> SubmitBodyGrokImagineBuilder {
        <SubmitBodyGrokImagineBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyGrokImagineBuilder {
    input: Option<InputGrokImagine>,
    webhook: Option<String>,
}

impl SubmitBodyGrokImagineBuilder {
    pub fn input(mut self, value: InputGrokImagine) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyGrokImagine`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyGrokImagineBuilder::input)
    pub fn build(self) -> Result<SubmitBodyGrokImagine, BuildError> {
        Ok(SubmitBodyGrokImagine {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

