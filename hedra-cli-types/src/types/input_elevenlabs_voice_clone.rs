pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `elevenlabs-voice-clone`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputElevenlabsVoiceClone {
    /// Speech sample of the voice to clone. Longer, cleaner samples of a single speaker clone better. At most 104.8 MB.
    pub audio: InputElevenlabsVoiceCloneAudio,
    /// Name for the created voice. Shown wherever the voice is listed; it does not affect how the voice sounds. At least 1 character.
    #[serde(default)]
    pub name: String,
}

impl InputElevenlabsVoiceClone {
    pub fn builder() -> InputElevenlabsVoiceCloneBuilder {
        <InputElevenlabsVoiceCloneBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputElevenlabsVoiceCloneBuilder {
    audio: Option<InputElevenlabsVoiceCloneAudio>,
    name: Option<String>,
}

impl InputElevenlabsVoiceCloneBuilder {
    pub fn audio(mut self, value: InputElevenlabsVoiceCloneAudio) -> Self {
        self.audio = Some(value);
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`InputElevenlabsVoiceClone`].
    /// This method will fail if any of the following fields are not set:
    /// - [`audio`](InputElevenlabsVoiceCloneBuilder::audio)
    /// - [`name`](InputElevenlabsVoiceCloneBuilder::name)
    pub fn build(self) -> Result<InputElevenlabsVoiceClone, BuildError> {
        Ok(InputElevenlabsVoiceClone {
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
        })
    }
}
