pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitBodyFlux11Pro {
    pub input: InputFlux11Pro,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFlux11Pro {
    pub fn builder() -> SubmitBodyFlux11ProBuilder {
        <SubmitBodyFlux11ProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFlux11ProBuilder {
    input: Option<InputFlux11Pro>,
    webhook: Option<String>,
}

impl SubmitBodyFlux11ProBuilder {
    pub fn input(mut self, value: InputFlux11Pro) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFlux11Pro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFlux11ProBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFlux11Pro, BuildError> {
        Ok(SubmitBodyFlux11Pro {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

