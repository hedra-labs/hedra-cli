pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyFluxKontextMax {
    #[serde(default)]
    pub input: InputFluxKontextMax,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFluxKontextMax {
    pub fn builder() -> SubmitBodyFluxKontextMaxBuilder {
        <SubmitBodyFluxKontextMaxBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFluxKontextMaxBuilder {
    input: Option<InputFluxKontextMax>,
    webhook: Option<String>,
}

impl SubmitBodyFluxKontextMaxBuilder {
    pub fn input(mut self, value: InputFluxKontextMax) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFluxKontextMax`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFluxKontextMaxBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFluxKontextMax, BuildError> {
        Ok(SubmitBodyFluxKontextMax {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

