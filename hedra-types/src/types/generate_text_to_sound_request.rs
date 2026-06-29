pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GenerateTextToSoundRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Optional agent thread ID to associate this generation with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_thread_id: Option<String>,
    /// Optional pre-reserved generation ID. If provided, this ID will be used instead of generating a new one. For batch operations (batch_size > 1), use generation_ids instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_id: Option<String>,
    /// Optional list of pre-reserved generation IDs for batch operations. Length must match batch_size. Mutually exclusive with generation_id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_ids: Option<Vec<String>>,
    /// The id of the model to use for sound effect generation.
    #[serde(default)]
    pub ai_model_id: String,
    /// The text description of the sound effect to generate.
    #[serde(default)]
    pub text: String,
    /// The duration of the sound effect in seconds (0.5-30). If not specified, duration is automatically determined from the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    /// Controls how strictly the model follows the prompt (0-1). Higher values mean more literal interpretation. Defaults to 0.3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_influence: Option<f64>,
    /// Whether to create a sound effect that loops smoothly. Only available for the eleven_text_to_sound_v2 model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#loop: Option<bool>,
    /// Output format of the generated audio. Formatted as codec_sample_rate_bitrate (e.g., mp3_22050_32, mp3_44100_128, pcm_44100). If not specified, defaults to mp3_44100_128.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
}

impl GenerateTextToSoundRequest {
    pub fn builder() -> GenerateTextToSoundRequestBuilder {
        <GenerateTextToSoundRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateTextToSoundRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    ai_model_id: Option<String>,
    text: Option<String>,
    duration_seconds: Option<f64>,
    prompt_influence: Option<f64>,
    r#loop: Option<bool>,
    output_format: Option<String>,
}

impl GenerateTextToSoundRequestBuilder {
    pub fn workspace_id(mut self, value: impl Into<String>) -> Self {
        self.workspace_id = Some(value.into());
        self
    }

    pub fn agent_thread_id(mut self, value: impl Into<String>) -> Self {
        self.agent_thread_id = Some(value.into());
        self
    }

    pub fn generation_id(mut self, value: impl Into<String>) -> Self {
        self.generation_id = Some(value.into());
        self
    }

    pub fn generation_ids(mut self, value: Vec<String>) -> Self {
        self.generation_ids = Some(value);
        self
    }

    pub fn ai_model_id(mut self, value: impl Into<String>) -> Self {
        self.ai_model_id = Some(value.into());
        self
    }

    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = Some(value.into());
        self
    }

    pub fn duration_seconds(mut self, value: f64) -> Self {
        self.duration_seconds = Some(value);
        self
    }

    pub fn prompt_influence(mut self, value: f64) -> Self {
        self.prompt_influence = Some(value);
        self
    }

    pub fn r#loop(mut self, value: bool) -> Self {
        self.r#loop = Some(value);
        self
    }

    pub fn output_format(mut self, value: impl Into<String>) -> Self {
        self.output_format = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`GenerateTextToSoundRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`ai_model_id`](GenerateTextToSoundRequestBuilder::ai_model_id)
    /// - [`text`](GenerateTextToSoundRequestBuilder::text)
    pub fn build(self) -> Result<GenerateTextToSoundRequest, BuildError> {
        Ok(GenerateTextToSoundRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            ai_model_id: self.ai_model_id.ok_or_else(|| BuildError::missing_field("ai_model_id"))?,
            text: self.text.ok_or_else(|| BuildError::missing_field("text"))?,
            duration_seconds: self.duration_seconds,
            prompt_influence: self.prompt_influence,
            r#loop: self.r#loop,
            output_format: self.output_format,
        })
    }
}
