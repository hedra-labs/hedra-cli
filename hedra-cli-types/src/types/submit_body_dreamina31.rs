pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitBodyDreamina31 {
    pub input: InputDreamina31,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyDreamina31 {
    pub fn builder() -> SubmitBodyDreamina31Builder {
        <SubmitBodyDreamina31Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyDreamina31Builder {
    input: Option<InputDreamina31>,
    webhook: Option<String>,
}

impl SubmitBodyDreamina31Builder {
    pub fn input(mut self, value: InputDreamina31) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyDreamina31`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyDreamina31Builder::input)
    pub fn build(self) -> Result<SubmitBodyDreamina31, BuildError> {
        Ok(SubmitBodyDreamina31 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

