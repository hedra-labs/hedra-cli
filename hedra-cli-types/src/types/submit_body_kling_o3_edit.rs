pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKlingO3Edit {
    pub input: InputKlingO3Edit,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyKlingO3Edit {
    pub fn builder() -> SubmitBodyKlingO3EditBuilder {
        <SubmitBodyKlingO3EditBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKlingO3EditBuilder {
    input: Option<InputKlingO3Edit>,
    webhook: Option<String>,
}

impl SubmitBodyKlingO3EditBuilder {
    pub fn input(mut self, value: InputKlingO3Edit) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyKlingO3Edit`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKlingO3EditBuilder::input)
    pub fn build(self) -> Result<SubmitBodyKlingO3Edit, BuildError> {
        Ok(SubmitBodyKlingO3Edit {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

