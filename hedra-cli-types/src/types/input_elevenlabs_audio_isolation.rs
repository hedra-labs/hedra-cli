pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `elevenlabs-audio-isolation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputElevenlabsAudioIsolation {
    /// Audio to strip background noise from. At most 104.8 MB.
    pub audio: InputElevenlabsAudioIsolationAudio,
}

impl InputElevenlabsAudioIsolation {
    pub fn builder() -> InputElevenlabsAudioIsolationBuilder {
        <InputElevenlabsAudioIsolationBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputElevenlabsAudioIsolationBuilder {
    audio: Option<InputElevenlabsAudioIsolationAudio>,
}

impl InputElevenlabsAudioIsolationBuilder {
    pub fn audio(mut self, value: InputElevenlabsAudioIsolationAudio) -> Self {
        self.audio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputElevenlabsAudioIsolation`].
    /// This method will fail if any of the following fields are not set:
    /// - [`audio`](InputElevenlabsAudioIsolationBuilder::audio)
    pub fn build(self) -> Result<InputElevenlabsAudioIsolation, BuildError> {
        Ok(InputElevenlabsAudioIsolation {
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
        })
    }
}
