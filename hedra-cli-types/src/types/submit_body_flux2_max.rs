pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyFlux2Max {
    pub input: InputFlux2Max,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFlux2Max {
    pub fn builder() -> SubmitBodyFlux2MaxBuilder {
        <SubmitBodyFlux2MaxBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFlux2MaxBuilder {
    input: Option<InputFlux2Max>,
    webhook: Option<String>,
}

impl SubmitBodyFlux2MaxBuilder {
    pub fn input(mut self, value: InputFlux2Max) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFlux2Max`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFlux2MaxBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFlux2Max, BuildError> {
        Ok(SubmitBodyFlux2Max {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

