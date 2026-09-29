pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyFluxDev {
    pub input: InputFluxDev,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFluxDev {
    pub fn builder() -> SubmitBodyFluxDevBuilder {
        <SubmitBodyFluxDevBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFluxDevBuilder {
    input: Option<InputFluxDev>,
    webhook: Option<String>,
}

impl SubmitBodyFluxDevBuilder {
    pub fn input(mut self, value: InputFluxDev) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFluxDev`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFluxDevBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFluxDev, BuildError> {
        Ok(SubmitBodyFluxDev {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

