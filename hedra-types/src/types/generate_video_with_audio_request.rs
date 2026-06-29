pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Video generation request that adds synchronized sound effects to video using Mirelo Studio.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GenerateVideoWithAudioRequest {
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
    /// ID of the model to use for video-to-video with audio generation (Mirelo Studio).
    #[serde(default)]
    pub video_generation_model_id: String,
    /// The id of the video asset to add sound effects to.
    #[serde(default)]
    pub video_id: String,
    /// Optional prompt to guide the audio generation for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

impl GenerateVideoWithAudioRequest {
    pub fn builder() -> GenerateVideoWithAudioRequestBuilder {
        <GenerateVideoWithAudioRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateVideoWithAudioRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    video_generation_model_id: Option<String>,
    video_id: Option<String>,
    prompt: Option<String>,
}

impl GenerateVideoWithAudioRequestBuilder {
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

    pub fn video_generation_model_id(mut self, value: impl Into<String>) -> Self {
        self.video_generation_model_id = Some(value.into());
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

    /// Consumes the builder and constructs a [`GenerateVideoWithAudioRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`video_generation_model_id`](GenerateVideoWithAudioRequestBuilder::video_generation_model_id)
    /// - [`video_id`](GenerateVideoWithAudioRequestBuilder::video_id)
    pub fn build(self) -> Result<GenerateVideoWithAudioRequest, BuildError> {
        Ok(GenerateVideoWithAudioRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            video_generation_model_id: self.video_generation_model_id.ok_or_else(|| BuildError::missing_field("video_generation_model_id"))?,
            video_id: self.video_id.ok_or_else(|| BuildError::missing_field("video_id"))?,
            prompt: self.prompt,
        })
    }
}
