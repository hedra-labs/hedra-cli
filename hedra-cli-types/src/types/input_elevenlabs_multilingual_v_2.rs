pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `elevenlabs-multilingual-v2`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputElevenlabsMultilingualV2 {
    /// Generation prompt. At most 10000 characters.
    #[serde(default)]
    pub text: String,
    /// Voice stability. Higher values give a steadier, more consistent delivery; lower values allow more expressive variation between generations. From 0 to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub stability: Option<f64>,
    /// Speech rate multiplier; 1.0 is the voice's natural pace. From 0.7 to 1.2.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub speed: Option<f64>,
    /// Language code; 'auto' by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Voice to speak with (`voice_<uuid>`). List valid voices at GET /v3/models/elevenlabs-multilingual-v2/voices.
    #[serde(default)]
    pub voice_id: String,
}

impl InputElevenlabsMultilingualV2 {
    pub fn builder() -> InputElevenlabsMultilingualV2Builder {
        <InputElevenlabsMultilingualV2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputElevenlabsMultilingualV2Builder {
    text: Option<String>,
    stability: Option<f64>,
    speed: Option<f64>,
    language: Option<String>,
    voice_id: Option<String>,
}

impl InputElevenlabsMultilingualV2Builder {
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = Some(value.into());
        self
    }

    pub fn stability(mut self, value: f64) -> Self {
        self.stability = Some(value);
        self
    }

    pub fn speed(mut self, value: f64) -> Self {
        self.speed = Some(value);
        self
    }

    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }

    pub fn voice_id(mut self, value: impl Into<String>) -> Self {
        self.voice_id = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`InputElevenlabsMultilingualV2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`text`](InputElevenlabsMultilingualV2Builder::text)
    /// - [`voice_id`](InputElevenlabsMultilingualV2Builder::voice_id)
    pub fn build(self) -> Result<InputElevenlabsMultilingualV2, BuildError> {
        Ok(InputElevenlabsMultilingualV2 {
            text: self.text.ok_or_else(|| BuildError::missing_field("text"))?,
            stability: self.stability,
            speed: self.speed,
            language: self.language,
            voice_id: self.voice_id.ok_or_else(|| BuildError::missing_field("voice_id"))?,
        })
    }
}
