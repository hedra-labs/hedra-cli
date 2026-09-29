pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBodyElevenlabsSoundEffects {
    #[serde(default)]
    pub input: InputElevenlabsSoundEffects,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyElevenlabsSoundEffects {
    pub fn builder() -> SubmitBodyElevenlabsSoundEffectsBuilder {
        <SubmitBodyElevenlabsSoundEffectsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsSoundEffectsBuilder {
    input: Option<InputElevenlabsSoundEffects>,
    webhook: Option<String>,
}

impl SubmitBodyElevenlabsSoundEffectsBuilder {
    pub fn input(mut self, value: InputElevenlabsSoundEffects) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsSoundEffects`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsSoundEffectsBuilder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsSoundEffects, BuildError> {
        Ok(SubmitBodyElevenlabsSoundEffects {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

