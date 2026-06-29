pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Video-to-video edit request for Kling O1 Edit model.
/// Transforms videos using natural language while preserving motion.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GenerateVideoToVideoRequest {
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
    /// The id of the model to use for video-to-video generation.
    #[serde(default)]
    pub ai_model_id: String,
    /// The id of the video asset to transform.
    #[serde(default)]
    pub video_id: String,
    /// Natural language transformation instructions. Use @Element1, @Image1 syntax to reference elements and images.
    #[serde(default)]
    pub prompt: String,
    /// Optional ids of reference image assets for style transfer. Reference as @Image1, @Image2, etc. in prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_image_asset_ids: Option<Vec<String>>,
    /// Optional elements for character tracking. Reference as @Element1, @Element2, etc. in prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<KlingO1EditElement>>,
    /// Whether to preserve the original audio from the input video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_audio: Option<bool>,
}

impl GenerateVideoToVideoRequest {
    pub fn builder() -> GenerateVideoToVideoRequestBuilder {
        <GenerateVideoToVideoRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateVideoToVideoRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    ai_model_id: Option<String>,
    video_id: Option<String>,
    prompt: Option<String>,
    reference_image_asset_ids: Option<Vec<String>>,
    elements: Option<Vec<KlingO1EditElement>>,
    keep_audio: Option<bool>,
}

impl GenerateVideoToVideoRequestBuilder {
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

    pub fn video_id(mut self, value: impl Into<String>) -> Self {
        self.video_id = Some(value.into());
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn reference_image_asset_ids(mut self, value: Vec<String>) -> Self {
        self.reference_image_asset_ids = Some(value);
        self
    }

    pub fn elements(mut self, value: Vec<KlingO1EditElement>) -> Self {
        self.elements = Some(value);
        self
    }

    pub fn keep_audio(mut self, value: bool) -> Self {
        self.keep_audio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GenerateVideoToVideoRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`ai_model_id`](GenerateVideoToVideoRequestBuilder::ai_model_id)
    /// - [`video_id`](GenerateVideoToVideoRequestBuilder::video_id)
    /// - [`prompt`](GenerateVideoToVideoRequestBuilder::prompt)
    pub fn build(self) -> Result<GenerateVideoToVideoRequest, BuildError> {
        Ok(GenerateVideoToVideoRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            ai_model_id: self.ai_model_id.ok_or_else(|| BuildError::missing_field("ai_model_id"))?,
            video_id: self.video_id.ok_or_else(|| BuildError::missing_field("video_id"))?,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            reference_image_asset_ids: self.reference_image_asset_ids,
            elements: self.elements,
            keep_audio: self.keep_audio,
        })
    }
}
