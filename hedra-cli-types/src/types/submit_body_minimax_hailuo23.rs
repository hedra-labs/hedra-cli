pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyMinimaxHailuo23 {
    #[serde(default)]
    pub input: InputMinimaxHailuo23,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMinimaxHailuo23 {
    pub fn builder() -> SubmitBodyMinimaxHailuo23Builder {
        <SubmitBodyMinimaxHailuo23Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxHailuo23Builder {
    input: Option<InputMinimaxHailuo23>,
    webhook: Option<String>,
}

impl SubmitBodyMinimaxHailuo23Builder {
    pub fn input(mut self, value: InputMinimaxHailuo23) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxHailuo23`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxHailuo23Builder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxHailuo23, BuildError> {
        Ok(SubmitBodyMinimaxHailuo23 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

