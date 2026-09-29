pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitBodyFlux11Ultra {
    pub input: InputFlux11Ultra,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFlux11Ultra {
    pub fn builder() -> SubmitBodyFlux11UltraBuilder {
        <SubmitBodyFlux11UltraBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFlux11UltraBuilder {
    input: Option<InputFlux11Ultra>,
    webhook: Option<String>,
}

impl SubmitBodyFlux11UltraBuilder {
    pub fn input(mut self, value: InputFlux11Ultra) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFlux11Ultra`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFlux11UltraBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFlux11Ultra, BuildError> {
        Ok(SubmitBodyFlux11Ultra {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

