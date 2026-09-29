pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKlingO3 {
    pub input: InputKlingO3,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyKlingO3 {
    pub fn builder() -> SubmitBodyKlingO3Builder {
        <SubmitBodyKlingO3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKlingO3Builder {
    input: Option<InputKlingO3>,
    webhook: Option<String>,
}

impl SubmitBodyKlingO3Builder {
    pub fn input(mut self, value: InputKlingO3) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyKlingO3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKlingO3Builder::input)
    pub fn build(self) -> Result<SubmitBodyKlingO3, BuildError> {
        Ok(SubmitBodyKlingO3 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

