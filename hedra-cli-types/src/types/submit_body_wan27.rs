pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyWan27 {
    pub input: InputWan27,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyWan27 {
    pub fn builder() -> SubmitBodyWan27Builder {
        <SubmitBodyWan27Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyWan27Builder {
    input: Option<InputWan27>,
    webhook: Option<String>,
}

impl SubmitBodyWan27Builder {
    pub fn input(mut self, value: InputWan27) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyWan27`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyWan27Builder::input)
    pub fn build(self) -> Result<SubmitBodyWan27, BuildError> {
        Ok(SubmitBodyWan27 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

