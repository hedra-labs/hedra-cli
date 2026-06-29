pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GeneratedImageInputs {
    /// Prompt for image generation.
    #[serde(default)]
    pub text_prompt: String,
    /// The id of the model used for generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model_id: Option<String>,
    /// Aspect ratio used for generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Resolution used for generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// The id of the Image asset used as the start keyframe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_keyframe_id: Option<String>,
    /// The id(s) of the image(s) referenced in the generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_image_ids: Option<Vec<String>>,
}

impl GeneratedImageInputs {
    pub fn builder() -> GeneratedImageInputsBuilder {
        <GeneratedImageInputsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GeneratedImageInputsBuilder {
    text_prompt: Option<String>,
    ai_model_id: Option<String>,
    aspect_ratio: Option<String>,
    resolution: Option<String>,
    start_keyframe_id: Option<String>,
    reference_image_ids: Option<Vec<String>>,
}

impl GeneratedImageInputsBuilder {
    pub fn text_prompt(mut self, value: impl Into<String>) -> Self {
        self.text_prompt = Some(value.into());
        self
    }

    pub fn ai_model_id(mut self, value: impl Into<String>) -> Self {
        self.ai_model_id = Some(value.into());
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

    pub fn reference_image_ids(mut self, value: Vec<String>) -> Self {
        self.reference_image_ids = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GeneratedImageInputs`].
    /// This method will fail if any of the following fields are not set:
    /// - [`text_prompt`](GeneratedImageInputsBuilder::text_prompt)
    pub fn build(self) -> Result<GeneratedImageInputs, BuildError> {
        Ok(GeneratedImageInputs {
            text_prompt: self.text_prompt.ok_or_else(|| BuildError::missing_field("text_prompt"))?,
            ai_model_id: self.ai_model_id,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            start_keyframe_id: self.start_keyframe_id,
            reference_image_ids: self.reference_image_ids,
        })
    }
}
