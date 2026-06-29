pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GenerateImageUpscaleRequest {
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
    /// The model to use for upscaling.
    #[serde(default)]
    pub ai_model_id: String,
    /// The id of the Image asset to use as the basis for upscaling.
    #[serde(default)]
    pub image_id: String,
    /// Optional upscale factor to pass to the model (e.g. 2.0 for 2x).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upscale_factor: Option<f64>,
}

impl GenerateImageUpscaleRequest {
    pub fn builder() -> GenerateImageUpscaleRequestBuilder {
        <GenerateImageUpscaleRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateImageUpscaleRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    ai_model_id: Option<String>,
    image_id: Option<String>,
    upscale_factor: Option<f64>,
}

impl GenerateImageUpscaleRequestBuilder {
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

    pub fn image_id(mut self, value: impl Into<String>) -> Self {
        self.image_id = Some(value.into());
        self
    }

    pub fn upscale_factor(mut self, value: f64) -> Self {
        self.upscale_factor = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GenerateImageUpscaleRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`ai_model_id`](GenerateImageUpscaleRequestBuilder::ai_model_id)
    /// - [`image_id`](GenerateImageUpscaleRequestBuilder::image_id)
    pub fn build(self) -> Result<GenerateImageUpscaleRequest, BuildError> {
        Ok(GenerateImageUpscaleRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            ai_model_id: self.ai_model_id.ok_or_else(|| BuildError::missing_field("ai_model_id"))?,
            image_id: self.image_id.ok_or_else(|| BuildError::missing_field("image_id"))?,
            upscale_factor: self.upscale_factor,
        })
    }
}
