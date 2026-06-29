pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerateImageResponse {
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
    /// The id of the resulting image asset.
    #[serde(default)]
    pub asset_id: String,
    /// The id of the generation. Can be used to check status.
    #[serde(default)]
    pub id: String,
    /// Date the generation was submitted.
    #[serde(default)]
    pub created_at: String,
    /// Status of the generation
    pub status: GenerationStatus,
    /// Current progress to completion. Between 0-1
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub progress: f64,
    /// Estimated time until completion in seconds. May be None if no historical data available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_sec: Option<i64>,
    /// Unique identifier linking all generations in a batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_generation_id: Option<String>,
    /// All generation results in the batch. Always populated (even for batch_size=1). The main response fields (id, asset_id, etc.) reflect the first successful generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_results: Option<Vec<BatchImageResultItem>>,
}

impl GenerateImageResponse {
    pub fn builder() -> GenerateImageResponseBuilder {
        <GenerateImageResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateImageResponseBuilder {
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
    asset_id: Option<String>,
    id: Option<String>,
    created_at: Option<String>,
    status: Option<GenerationStatus>,
    progress: Option<f64>,
    eta_sec: Option<i64>,
    batch_generation_id: Option<String>,
    batch_results: Option<Vec<BatchImageResultItem>>,
}

impl GenerateImageResponseBuilder {
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

    pub fn asset_id(mut self, value: impl Into<String>) -> Self {
        self.asset_id = Some(value.into());
        self
    }

    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: impl Into<String>) -> Self {
        self.created_at = Some(value.into());
        self
    }

    pub fn status(mut self, value: GenerationStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn progress(mut self, value: f64) -> Self {
        self.progress = Some(value);
        self
    }

    pub fn eta_sec(mut self, value: i64) -> Self {
        self.eta_sec = Some(value);
        self
    }

    pub fn batch_generation_id(mut self, value: impl Into<String>) -> Self {
        self.batch_generation_id = Some(value.into());
        self
    }

    pub fn batch_results(mut self, value: Vec<BatchImageResultItem>) -> Self {
        self.batch_results = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GenerateImageResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`text_prompt`](GenerateImageResponseBuilder::text_prompt)
    /// - [`ai_model_id`](GenerateImageResponseBuilder::ai_model_id)
    /// - [`asset_id`](GenerateImageResponseBuilder::asset_id)
    /// - [`id`](GenerateImageResponseBuilder::id)
    /// - [`created_at`](GenerateImageResponseBuilder::created_at)
    /// - [`status`](GenerateImageResponseBuilder::status)
    /// - [`progress`](GenerateImageResponseBuilder::progress)
    pub fn build(self) -> Result<GenerateImageResponse, BuildError> {
        Ok(GenerateImageResponse {
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
            asset_id: self.asset_id.ok_or_else(|| BuildError::missing_field("asset_id"))?,
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            progress: self.progress.ok_or_else(|| BuildError::missing_field("progress"))?,
            eta_sec: self.eta_sec,
            batch_generation_id: self.batch_generation_id,
            batch_results: self.batch_results,
        })
    }
}
