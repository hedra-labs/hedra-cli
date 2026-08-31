pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `elevenlabs-music`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct InputElevenlabsMusic {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Length of the generated track, in milliseconds. Billed as whole seconds, rounded up. From 3000 to 600000.
    #[serde(default)]
    pub duration_ms: i64,
    /// Words for the track to sing. Sent to the provider inside the prompt, which reads lyrics from the prompt text and places them across the requested length. Cannot be combined with `force_instrumental`. At most 10000 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    /// Compose without vocals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_instrumental: Option<bool>,
}

impl InputElevenlabsMusic {
    pub fn builder() -> InputElevenlabsMusicBuilder {
        <InputElevenlabsMusicBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputElevenlabsMusicBuilder {
    prompt: Option<String>,
    duration_ms: Option<i64>,
    lyrics: Option<String>,
    force_instrumental: Option<bool>,
}

impl InputElevenlabsMusicBuilder {
    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn lyrics(mut self, value: impl Into<String>) -> Self {
        self.lyrics = Some(value.into());
        self
    }

    pub fn force_instrumental(mut self, value: bool) -> Self {
        self.force_instrumental = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputElevenlabsMusic`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputElevenlabsMusicBuilder::prompt)
    /// - [`duration_ms`](InputElevenlabsMusicBuilder::duration_ms)
    pub fn build(self) -> Result<InputElevenlabsMusic, BuildError> {
        Ok(InputElevenlabsMusic {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            lyrics: self.lyrics,
            force_instrumental: self.force_instrumental,
        })
    }
}
