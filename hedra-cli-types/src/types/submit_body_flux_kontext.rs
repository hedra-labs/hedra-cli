pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyFluxKontext {
    #[serde(default)]
    pub input: InputFluxKontext,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFluxKontext {
    pub fn builder() -> SubmitBodyFluxKontextBuilder {
        <SubmitBodyFluxKontextBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFluxKontextBuilder {
    input: Option<InputFluxKontext>,
    webhook: Option<String>,
}

impl SubmitBodyFluxKontextBuilder {
    pub fn input(mut self, value: InputFluxKontext) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFluxKontext`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFluxKontextBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFluxKontext, BuildError> {
        Ok(SubmitBodyFluxKontext {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

