pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyMinimaxH3Ultra {
    #[serde(default)]
    pub input: InputMinimaxH3Ultra,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMinimaxH3Ultra {
    pub fn builder() -> SubmitBodyMinimaxH3UltraBuilder {
        <SubmitBodyMinimaxH3UltraBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxH3UltraBuilder {
    input: Option<InputMinimaxH3Ultra>,
    webhook: Option<String>,
}

impl SubmitBodyMinimaxH3UltraBuilder {
    pub fn input(mut self, value: InputMinimaxH3Ultra) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxH3Ultra`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxH3UltraBuilder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxH3Ultra, BuildError> {
        Ok(SubmitBodyMinimaxH3Ultra {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

