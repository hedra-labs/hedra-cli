pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SubmitBodyElevenlabsMusic {
    #[serde(default)]
    pub input: InputElevenlabsMusic,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyElevenlabsMusic {
    pub fn builder() -> SubmitBodyElevenlabsMusicBuilder {
        <SubmitBodyElevenlabsMusicBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsMusicBuilder {
    input: Option<InputElevenlabsMusic>,
    webhook: Option<String>,
}

impl SubmitBodyElevenlabsMusicBuilder {
    pub fn input(mut self, value: InputElevenlabsMusic) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsMusic`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsMusicBuilder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsMusic, BuildError> {
        Ok(SubmitBodyElevenlabsMusic {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

