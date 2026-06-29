pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct VoiceLabel {
    /// Label name.
    #[serde(default)]
    pub name: String,
    /// Label value.
    #[serde(default)]
    pub value: String,
}

impl VoiceLabel {
    pub fn builder() -> VoiceLabelBuilder {
        <VoiceLabelBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct VoiceLabelBuilder {
    name: Option<String>,
    value: Option<String>,
}

impl VoiceLabelBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`VoiceLabel`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](VoiceLabelBuilder::name)
    /// - [`value`](VoiceLabelBuilder::value)
    pub fn build(self) -> Result<VoiceLabel, BuildError> {
        Ok(VoiceLabel {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            value: self.value.ok_or_else(|| BuildError::missing_field("value"))?,
        })
    }
}
