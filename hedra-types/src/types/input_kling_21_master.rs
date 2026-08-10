pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-21-master`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, start_image
/// (2) requires: aspect_ratio, duration_ms, prompt; must omit: start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKling21Master {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputKling21MasterAspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKling21MasterResolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputKling21MasterStartImage>,
    /// What to avoid in the generated video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// How closely the model follows the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub cfg_scale: Option<f64>,
}

impl InputKling21Master {
    pub fn builder() -> InputKling21MasterBuilder {
        <InputKling21MasterBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKling21MasterBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputKling21MasterAspectRatio>,
    resolution: Option<InputKling21MasterResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputKling21MasterStartImage>,
    negative_prompt: Option<String>,
    cfg_scale: Option<f64>,
}

impl InputKling21MasterBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputKling21MasterAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKling21MasterResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKling21MasterStartImage) -> Self {
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

    /// Consumes the builder and constructs a [`InputKling21Master`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKling21MasterBuilder::prompt)
    /// - [`aspect_ratio`](InputKling21MasterBuilder::aspect_ratio)
    /// - [`duration_ms`](InputKling21MasterBuilder::duration_ms)
    pub fn build(self) -> Result<InputKling21Master, BuildError> {
        Ok(InputKling21Master {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            negative_prompt: self.negative_prompt,
            cfg_scale: self.cfg_scale,
        })
    }
}
