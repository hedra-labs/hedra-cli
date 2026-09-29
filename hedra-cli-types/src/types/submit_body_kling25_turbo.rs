pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKling25Turbo {
    pub input: InputKling25Turbo,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyKling25Turbo {
    pub fn builder() -> SubmitBodyKling25TurboBuilder {
        <SubmitBodyKling25TurboBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKling25TurboBuilder {
    input: Option<InputKling25Turbo>,
    webhook: Option<String>,
}

impl SubmitBodyKling25TurboBuilder {
    pub fn input(mut self, value: InputKling25Turbo) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyKling25Turbo`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKling25TurboBuilder::input)
    pub fn build(self) -> Result<SubmitBodyKling25Turbo, BuildError> {
        Ok(SubmitBodyKling25Turbo {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

