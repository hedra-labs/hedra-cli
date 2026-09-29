pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyMinimaxH3Max {
    pub input: InputMinimaxH3Max,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMinimaxH3Max {
    pub fn builder() -> SubmitBodyMinimaxH3MaxBuilder {
        <SubmitBodyMinimaxH3MaxBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxH3MaxBuilder {
    input: Option<InputMinimaxH3Max>,
    webhook: Option<String>,
}

impl SubmitBodyMinimaxH3MaxBuilder {
    pub fn input(mut self, value: InputMinimaxH3Max) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxH3Max`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxH3MaxBuilder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxH3Max, BuildError> {
        Ok(SubmitBodyMinimaxH3Max {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

