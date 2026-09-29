pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyKling26MotionControl {
    pub input: InputKling26MotionControl,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyKling26MotionControl {
    pub fn builder() -> SubmitBodyKling26MotionControlBuilder {
        <SubmitBodyKling26MotionControlBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyKling26MotionControlBuilder {
    input: Option<InputKling26MotionControl>,
    webhook: Option<String>,
}

impl SubmitBodyKling26MotionControlBuilder {
    pub fn input(mut self, value: InputKling26MotionControl) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyKling26MotionControl`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyKling26MotionControlBuilder::input)
    pub fn build(self) -> Result<SubmitBodyKling26MotionControl, BuildError> {
        Ok(SubmitBodyKling26MotionControl {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

