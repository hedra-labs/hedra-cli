pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `elevenlabs-english-sts-v2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputElevenlabsEnglishStsV2 {
    /// Speech to re-voice. The delivery, pacing, and timing of this recording are preserved; only the voice changes. At most 104.8 MB.
    pub audio: InputElevenlabsEnglishStsV2Audio,
    /// Voice to speak with (`voice_<uuid>`). List the voices you can use at GET /v3/models/elevenlabs-english-sts-v2/voices; sent with your credentials, the list includes the voices you cloned.
    #[serde(default)]
    pub voice_id: String,
}

impl InputElevenlabsEnglishStsV2 {
    pub fn builder() -> InputElevenlabsEnglishStsV2Builder {
        <InputElevenlabsEnglishStsV2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputElevenlabsEnglishStsV2Builder {
    audio: Option<InputElevenlabsEnglishStsV2Audio>,
    voice_id: Option<String>,
}

impl InputElevenlabsEnglishStsV2Builder {
    pub fn audio(mut self, value: InputElevenlabsEnglishStsV2Audio) -> Self {
        self.audio = Some(value);
        self
    }

    pub fn voice_id(mut self, value: impl Into<String>) -> Self {
        self.voice_id = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`InputElevenlabsEnglishStsV2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`audio`](InputElevenlabsEnglishStsV2Builder::audio)
    /// - [`voice_id`](InputElevenlabsEnglishStsV2Builder::voice_id)
    pub fn build(self) -> Result<InputElevenlabsEnglishStsV2, BuildError> {
        Ok(InputElevenlabsEnglishStsV2 {
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
            voice_id: self.voice_id.ok_or_else(|| BuildError::missing_field("voice_id"))?,
        })
    }
}
