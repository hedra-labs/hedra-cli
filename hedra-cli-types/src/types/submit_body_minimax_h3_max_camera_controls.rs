pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyMinimaxH3MaxCameraControls {
    pub input: InputMinimaxH3MaxCameraControls,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMinimaxH3MaxCameraControls {
    pub fn builder() -> SubmitBodyMinimaxH3MaxCameraControlsBuilder {
        <SubmitBodyMinimaxH3MaxCameraControlsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMinimaxH3MaxCameraControlsBuilder {
    input: Option<InputMinimaxH3MaxCameraControls>,
    webhook: Option<String>,
}

impl SubmitBodyMinimaxH3MaxCameraControlsBuilder {
    pub fn input(mut self, value: InputMinimaxH3MaxCameraControls) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMinimaxH3MaxCameraControls`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMinimaxH3MaxCameraControlsBuilder::input)
    pub fn build(self) -> Result<SubmitBodyMinimaxH3MaxCameraControls, BuildError> {
        Ok(SubmitBodyMinimaxH3MaxCameraControls {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

