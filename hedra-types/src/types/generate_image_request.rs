pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GenerateImageRequest {
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
    /// The text prompt for image generation or image editing.
    #[serde(default)]
    pub text_prompt: String,
    /// The aspect ratio to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// The resolution to use formatted like '540p', '1080p', '1440p (2K QHD)', etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// The id of the Image asset to use as the start keyframe. This will be ignored if reference_image_ids is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_keyframe_id: Option<String>,
    /// The model to use.
    #[serde(default)]
    pub ai_model_id: String,
    /// The id(s) of the image(s) to reference in the generation. This is only used for image-to-image generation and will supersede start_keyframe_id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_image_ids: Option<Vec<String>>,
    /// Number of image variations to generate (1-8). When > 1, batch_results will contain all generation results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<i64>,
    /// If true, automatically enhance the prompt before generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
}

impl GenerateImageRequest {
    pub fn builder() -> GenerateImageRequestBuilder {
        <GenerateImageRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateImageRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    text_prompt: Option<String>,
    aspect_ratio: Option<String>,
    resolution: Option<String>,
    start_keyframe_id: Option<String>,
    ai_model_id: Option<String>,
    reference_image_ids: Option<Vec<String>>,
    batch_size: Option<i64>,
    enhance_prompt: Option<bool>,
}

impl GenerateImageRequestBuilder {
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

    pub fn text_prompt(mut self, value: impl Into<String>) -> Self {
        self.text_prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: impl Into<String>) -> Self {
        self.aspect_ratio = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: impl Into<String>) -> Self {
        self.resolution = Some(value.into());
        self
    }

    pub fn start_keyframe_id(mut self, value: impl Into<String>) -> Self {
        self.start_keyframe_id = Some(value.into());
        self
    }

    pub fn ai_model_id(mut self, value: impl Into<String>) -> Self {
        self.ai_model_id = Some(value.into());
        self
    }

    pub fn reference_image_ids(mut self, value: Vec<String>) -> Self {
        self.reference_image_ids = Some(value);
        self
    }

    pub fn batch_size(mut self, value: i64) -> Self {
        self.batch_size = Some(value);
        self
    }

    pub fn enhance_prompt(mut self, value: bool) -> Self {
        self.enhance_prompt = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GenerateImageRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`text_prompt`](GenerateImageRequestBuilder::text_prompt)
    /// - [`ai_model_id`](GenerateImageRequestBuilder::ai_model_id)
    pub fn build(self) -> Result<GenerateImageRequest, BuildError> {
        Ok(GenerateImageRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            text_prompt: self.text_prompt.ok_or_else(|| BuildError::missing_field("text_prompt"))?,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            start_keyframe_id: self.start_keyframe_id,
            ai_model_id: self.ai_model_id.ok_or_else(|| BuildError::missing_field("ai_model_id"))?,
            reference_image_ids: self.reference_image_ids,
            batch_size: self.batch_size,
            enhance_prompt: self.enhance_prompt,
        })
    }
}
