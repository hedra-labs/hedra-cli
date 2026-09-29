pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodySana {
    pub input: InputSana,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodySana {
    pub fn builder() -> SubmitBodySanaBuilder {
        <SubmitBodySanaBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodySanaBuilder {
    input: Option<InputSana>,
    webhook: Option<String>,
}

impl SubmitBodySanaBuilder {
    pub fn input(mut self, value: InputSana) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodySana`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodySanaBuilder::input)
    pub fn build(self) -> Result<SubmitBodySana, BuildError> {
        Ok(SubmitBodySana {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

