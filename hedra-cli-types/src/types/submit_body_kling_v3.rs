pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKlingV3 {
    pub input: InputKlingV3,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyKlingV3 {
    pub fn builder() -> SubmitBodyKlingV3Builder {
        <SubmitBodyKlingV3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKlingV3Builder {
    input: Option<InputKlingV3>,
    webhook: Option<String>,
}

impl SubmitBodyKlingV3Builder {
    pub fn input(mut self, value: InputKlingV3) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyKlingV3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKlingV3Builder::input)
    pub fn build(self) -> Result<SubmitBodyKlingV3, BuildError> {
        Ok(SubmitBodyKlingV3 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

