pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GeneratedVideoInputs {
    /// Prompt for video generation.
    #[serde(default)]
    pub text_prompt: String,
    /// The id of the model used for generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model_id: Option<String>,
    /// Resolution for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Aspect ratio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Duration of the video in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Normalized coordinates for primary speaker position (Character3 only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box_target: Option<NormalizedPoint>,
    /// For motion control models: 'video' matches reference video orientation (better for complex motions, max 30s), 'image' preserves character image orientation (better for camera movements, max 10s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_orientation: Option<GeneratedVideoInputsCharacterOrientation>,
    /// If true, automatically enhance the prompt before generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
}

impl GeneratedVideoInputs {
    pub fn builder() -> GeneratedVideoInputsBuilder {
        <GeneratedVideoInputsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GeneratedVideoInputsBuilder {
    text_prompt: Option<String>,
    ai_model_id: Option<String>,
    resolution: Option<String>,
    aspect_ratio: Option<String>,
    duration_ms: Option<i64>,
    bounding_box_target: Option<NormalizedPoint>,
    character_orientation: Option<GeneratedVideoInputsCharacterOrientation>,
    enhance_prompt: Option<bool>,
}

impl GeneratedVideoInputsBuilder {
    pub fn text_prompt(mut self, value: impl Into<String>) -> Self {
        self.text_prompt = Some(value.into());
        self
    }

    pub fn ai_model_id(mut self, value: impl Into<String>) -> Self {
        self.ai_model_id = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: impl Into<String>) -> Self {
        self.resolution = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: impl Into<String>) -> Self {
        self.aspect_ratio = Some(value.into());
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn bounding_box_target(mut self, value: NormalizedPoint) -> Self {
        self.bounding_box_target = Some(value);
        self
    }

    pub fn character_orientation(mut self, value: GeneratedVideoInputsCharacterOrientation) -> Self {
        self.character_orientation = Some(value);
        self
    }

    pub fn enhance_prompt(mut self, value: bool) -> Self {
        self.enhance_prompt = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GeneratedVideoInputs`].
    /// This method will fail if any of the following fields are not set:
    /// - [`text_prompt`](GeneratedVideoInputsBuilder::text_prompt)
    pub fn build(self) -> Result<GeneratedVideoInputs, BuildError> {
        Ok(GeneratedVideoInputs {
            text_prompt: self.text_prompt.ok_or_else(|| BuildError::missing_field("text_prompt"))?,
            ai_model_id: self.ai_model_id,
            resolution: self.resolution,
            aspect_ratio: self.aspect_ratio,
            duration_ms: self.duration_ms,
            bounding_box_target: self.bounding_box_target,
            character_orientation: self.character_orientation,
            enhance_prompt: self.enhance_prompt,
        })
    }
}
