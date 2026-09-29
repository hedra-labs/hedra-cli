pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyPixverseV6 {
    pub input: InputPixverseV6,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyPixverseV6 {
    pub fn builder() -> SubmitBodyPixverseV6Builder {
        <SubmitBodyPixverseV6Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyPixverseV6Builder {
    input: Option<InputPixverseV6>,
    webhook: Option<String>,
}

impl SubmitBodyPixverseV6Builder {
    pub fn input(mut self, value: InputPixverseV6) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyPixverseV6`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyPixverseV6Builder::input)
    pub fn build(self) -> Result<SubmitBodyPixverseV6, BuildError> {
        Ok(SubmitBodyPixverseV6 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

