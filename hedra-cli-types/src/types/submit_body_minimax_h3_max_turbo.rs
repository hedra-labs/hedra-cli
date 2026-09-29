pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyMinimaxH3MaxTurbo {
    pub input: InputMinimaxH3MaxTurbo,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMinimaxH3MaxTurbo {
    pub fn builder() -> SubmitBodyMinimaxH3MaxTurboBuilder {
        <SubmitBodyMinimaxH3MaxTurboBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxH3MaxTurboBuilder {
    input: Option<InputMinimaxH3MaxTurbo>,
    webhook: Option<String>,
}

impl SubmitBodyMinimaxH3MaxTurboBuilder {
    pub fn input(mut self, value: InputMinimaxH3MaxTurbo) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxH3MaxTurbo`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxH3MaxTurboBuilder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxH3MaxTurbo, BuildError> {
        Ok(SubmitBodyMinimaxH3MaxTurbo {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

