pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyMinimaxHailuo02 {
    #[serde(default)]
    pub input: InputMinimaxHailuo02,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMinimaxHailuo02 {
    pub fn builder() -> SubmitBodyMinimaxHailuo02Builder {
        <SubmitBodyMinimaxHailuo02Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxHailuo02Builder {
    input: Option<InputMinimaxHailuo02>,
    webhook: Option<String>,
}

impl SubmitBodyMinimaxHailuo02Builder {
    pub fn input(mut self, value: InputMinimaxHailuo02) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxHailuo02`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxHailuo02Builder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxHailuo02, BuildError> {
        Ok(SubmitBodyMinimaxHailuo02 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

