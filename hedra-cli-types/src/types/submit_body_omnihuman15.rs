pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyOmnihuman15 {
    pub input: InputOmnihuman15,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyOmnihuman15 {
    pub fn builder() -> SubmitBodyOmnihuman15Builder {
        <SubmitBodyOmnihuman15Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyOmnihuman15Builder {
    input: Option<InputOmnihuman15>,
    webhook: Option<String>,
}

impl SubmitBodyOmnihuman15Builder {
    pub fn input(mut self, value: InputOmnihuman15) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyOmnihuman15`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyOmnihuman15Builder::input)
    pub fn build(self) -> Result<SubmitBodyOmnihuman15, BuildError> {
        Ok(SubmitBodyOmnihuman15 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

