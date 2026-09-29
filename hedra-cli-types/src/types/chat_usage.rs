pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The tokens a chat completion used, as the model reported them.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ChatUsage {
    /// Tokens in the prompt.
    #[serde(default)]
    pub prompt_tokens: i64,
    /// Tokens generated, reasoning tokens included.
    #[serde(default)]
    pub completion_tokens: i64,
    /// Of `completion_tokens`, how many the model spent reasoning; null when the model reports none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<i64>,
    /// True when the model reported no usage and the counts are estimated from the request and response bytes.
    #[serde(default)]
    pub estimated: bool,
}

impl ChatUsage {
    pub fn builder() -> ChatUsageBuilder {
        <ChatUsageBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ChatUsageBuilder {
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    reasoning_tokens: Option<i64>,
    estimated: Option<bool>,
}

impl ChatUsageBuilder {
    pub fn prompt_tokens(mut self, value: i64) -> Self {
        self.prompt_tokens = Some(value);
        self
    }

    pub fn completion_tokens(mut self, value: i64) -> Self {
        self.completion_tokens = Some(value);
        self
    }

    pub fn reasoning_tokens(mut self, value: i64) -> Self {
        self.reasoning_tokens = Some(value);
        self
    }

    pub fn estimated(mut self, value: bool) -> Self {
        self.estimated = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ChatUsage`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt_tokens`](ChatUsageBuilder::prompt_tokens)
    /// - [`completion_tokens`](ChatUsageBuilder::completion_tokens)
    /// - [`estimated`](ChatUsageBuilder::estimated)
    pub fn build(self) -> Result<ChatUsage, BuildError> {
        Ok(ChatUsage {
            prompt_tokens: self.prompt_tokens.ok_or_else(|| BuildError::missing_field("prompt_tokens"))?,
            completion_tokens: self.completion_tokens.ok_or_else(|| BuildError::missing_field("completion_tokens"))?,
            reasoning_tokens: self.reasoning_tokens,
            estimated: self.estimated.ok_or_else(|| BuildError::missing_field("estimated"))?,
        })
    }
}
