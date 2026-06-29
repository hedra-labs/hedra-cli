pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Audio generation request that extracts sound effects from video using Mirelo Studio.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GenerateAudioFromVideoRequest {
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
    /// ID of the model to use for video-to-audio generation (Mirelo Studio).
    #[serde(default)]
    pub audio_generation_model_id: String,
    /// The id of the video asset to generate audio from.
    #[serde(default)]
    pub video_id: String,
    /// Optional prompt to guide the audio generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

impl GenerateAudioFromVideoRequest {
    pub fn builder() -> GenerateAudioFromVideoRequestBuilder {
        <GenerateAudioFromVideoRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateAudioFromVideoRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    audio_generation_model_id: Option<String>,
    video_id: Option<String>,
    prompt: Option<String>,
}

impl GenerateAudioFromVideoRequestBuilder {
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

    pub fn audio_generation_model_id(mut self, value: impl Into<String>) -> Self {
        self.audio_generation_model_id = Some(value.into());
        self
    }

    pub fn video_id(mut self, value: impl Into<String>) -> Self {
        self.video_id = Some(value.into());
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`GenerateAudioFromVideoRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`audio_generation_model_id`](GenerateAudioFromVideoRequestBuilder::audio_generation_model_id)
    /// - [`video_id`](GenerateAudioFromVideoRequestBuilder::video_id)
    pub fn build(self) -> Result<GenerateAudioFromVideoRequest, BuildError> {
        Ok(GenerateAudioFromVideoRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            audio_generation_model_id: self.audio_generation_model_id.ok_or_else(|| BuildError::missing_field("audio_generation_model_id"))?,
            video_id: self.video_id.ok_or_else(|| BuildError::missing_field("video_id"))?,
            prompt: self.prompt,
        })
    }
}
