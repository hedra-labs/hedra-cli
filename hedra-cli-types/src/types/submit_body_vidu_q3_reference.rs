pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyViduQ3Reference {
    pub input: InputViduQ3Reference,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyViduQ3Reference {
    pub fn builder() -> SubmitBodyViduQ3ReferenceBuilder {
        <SubmitBodyViduQ3ReferenceBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyViduQ3ReferenceBuilder {
    input: Option<InputViduQ3Reference>,
    webhook: Option<String>,
}

impl SubmitBodyViduQ3ReferenceBuilder {
    pub fn input(mut self, value: InputViduQ3Reference) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyViduQ3Reference`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyViduQ3ReferenceBuilder::input)
    pub fn build(self) -> Result<SubmitBodyViduQ3Reference, BuildError> {
        Ok(SubmitBodyViduQ3Reference {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

