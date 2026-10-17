pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyNanoBanana21 {
    pub input: InputNanoBanana21,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyNanoBanana21 {
    pub fn builder() -> SubmitBodyNanoBanana21Builder {
        <SubmitBodyNanoBanana21Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyNanoBanana21Builder {
    input: Option<InputNanoBanana21>,
    webhook: Option<String>,
}

impl SubmitBodyNanoBanana21Builder {
    pub fn input(mut self, value: InputNanoBanana21) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyNanoBanana21`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyNanoBanana21Builder::input)
    pub fn build(self) -> Result<SubmitBodyNanoBanana21, BuildError> {
        Ok(SubmitBodyNanoBanana21 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

