pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitBodyReve21 {
    pub input: InputReve21,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyReve21 {
    pub fn builder() -> SubmitBodyReve21Builder {
        <SubmitBodyReve21Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyReve21Builder {
    input: Option<InputReve21>,
    webhook: Option<String>,
}

impl SubmitBodyReve21Builder {
    pub fn input(mut self, value: InputReve21) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyReve21`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyReve21Builder::input)
    pub fn build(self) -> Result<SubmitBodyReve21, BuildError> {
        Ok(SubmitBodyReve21 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

