pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyElevenlabsVoiceClone {
    pub input: InputElevenlabsVoiceClone,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyElevenlabsVoiceClone {
    pub fn builder() -> SubmitBodyElevenlabsVoiceCloneBuilder {
        <SubmitBodyElevenlabsVoiceCloneBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyElevenlabsVoiceCloneBuilder {
    input: Option<InputElevenlabsVoiceClone>,
    webhook: Option<String>,
}

impl SubmitBodyElevenlabsVoiceCloneBuilder {
    pub fn input(mut self, value: InputElevenlabsVoiceClone) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyElevenlabsVoiceClone`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyElevenlabsVoiceCloneBuilder::input)
    pub fn build(self) -> Result<SubmitBodyElevenlabsVoiceClone, BuildError> {
        Ok(SubmitBodyElevenlabsVoiceClone {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

