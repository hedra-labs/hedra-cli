pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-16`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: prompt, start_image; accepts resolution: 1080p
/// (2) requires: prompt; must omit: start_image; accepts resolution: 720p
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputKling16 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputKling16AspectRatio>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKling16Resolution>,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Start frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputKling16StartImage>,
    /// What to avoid in the generated video. At most 2500 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// How closely the model follows the prompt. From 0 to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub cfg_scale: Option<f64>,
}

impl InputKling16 {
    pub fn builder() -> InputKling16Builder {
        <InputKling16Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKling16Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputKling16AspectRatio>,
    resolution: Option<InputKling16Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputKling16StartImage>,
    negative_prompt: Option<String>,
    cfg_scale: Option<f64>,
}

impl InputKling16Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputKling16AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKling16Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKling16StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn cfg_scale(mut self, value: f64) -> Self {
        self.cfg_scale = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKling16`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKling16Builder::prompt)
    pub fn build(self) -> Result<InputKling16, BuildError> {
        Ok(InputKling16 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            duration_ms: self.duration_ms,
            start_image: self.start_image,
            negative_prompt: self.negative_prompt,
            cfg_scale: self.cfg_scale,
        })
    }
}
