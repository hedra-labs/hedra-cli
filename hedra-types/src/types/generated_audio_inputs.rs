pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GeneratedAudioInputs {
    /// Prompt for audio generation.
    #[serde(default)]
    pub text_prompt: String,
    /// The id of the model used for generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model_id: Option<String>,
    /// The id of the voice used for generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    /// Stability setting used for generation (0-1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f64>,
    /// Speed setting used for generation (0.7-1.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// Language used for generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl GeneratedAudioInputs {
    pub fn builder() -> GeneratedAudioInputsBuilder {
        <GeneratedAudioInputsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GeneratedAudioInputsBuilder {
    text_prompt: Option<String>,
    ai_model_id: Option<String>,
    voice_id: Option<String>,
    stability: Option<f64>,
    speed: Option<f64>,
    language: Option<String>,
}

impl GeneratedAudioInputsBuilder {
    pub fn text_prompt(mut self, value: impl Into<String>) -> Self {
        self.text_prompt = Some(value.into());
        self
    }

    pub fn ai_model_id(mut self, value: impl Into<String>) -> Self {
        self.ai_model_id = Some(value.into());
        self
    }

    pub fn voice_id(mut self, value: impl Into<String>) -> Self {
        self.voice_id = Some(value.into());
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

    /// Consumes the builder and constructs a [`GeneratedAudioInputs`].
    /// This method will fail if any of the following fields are not set:
    /// - [`text_prompt`](GeneratedAudioInputsBuilder::text_prompt)
    pub fn build(self) -> Result<GeneratedAudioInputs, BuildError> {
        Ok(GeneratedAudioInputs {
            text_prompt: self.text_prompt.ok_or_else(|| BuildError::missing_field("text_prompt"))?,
            ai_model_id: self.ai_model_id,
            voice_id: self.voice_id,
            stability: self.stability,
            speed: self.speed,
            language: self.language,
        })
    }
}
