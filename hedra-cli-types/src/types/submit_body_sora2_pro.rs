pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodySora2Pro {
    pub input: InputSora2Pro,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodySora2Pro {
    pub fn builder() -> SubmitBodySora2ProBuilder {
        <SubmitBodySora2ProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodySora2ProBuilder {
    input: Option<InputSora2Pro>,
    webhook: Option<String>,
}

impl SubmitBodySora2ProBuilder {
    pub fn input(mut self, value: InputSora2Pro) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodySora2Pro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodySora2ProBuilder::input)
    pub fn build(self) -> Result<SubmitBodySora2Pro, BuildError> {
        Ok(SubmitBodySora2Pro {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

