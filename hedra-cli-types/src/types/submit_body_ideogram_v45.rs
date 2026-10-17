pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyIdeogramV45 {
    #[serde(default)]
    pub input: InputIdeogramV45,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyIdeogramV45 {
    pub fn builder() -> SubmitBodyIdeogramV45Builder {
        <SubmitBodyIdeogramV45Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyIdeogramV45Builder {
    input: Option<InputIdeogramV45>,
    webhook: Option<String>,
}

impl SubmitBodyIdeogramV45Builder {
    pub fn input(mut self, value: InputIdeogramV45) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyIdeogramV45`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyIdeogramV45Builder::input)
    pub fn build(self) -> Result<SubmitBodyIdeogramV45, BuildError> {
        Ok(SubmitBodyIdeogramV45 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

