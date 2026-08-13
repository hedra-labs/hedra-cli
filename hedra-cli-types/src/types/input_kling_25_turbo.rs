pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-25-turbo`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, start_image
/// (2) requires: aspect_ratio, duration_ms, prompt; must omit: start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKling25Turbo {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputKling25TurboAspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKling25TurboResolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video). At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputKling25TurboStartImage>,
    /// What to avoid in the generated video. At most 2500 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// How closely the model follows the prompt. From 0 to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub cfg_scale: Option<f64>,
}

impl InputKling25Turbo {
    pub fn builder() -> InputKling25TurboBuilder {
        <InputKling25TurboBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKling25TurboBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputKling25TurboAspectRatio>,
    resolution: Option<InputKling25TurboResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputKling25TurboStartImage>,
    negative_prompt: Option<String>,
    cfg_scale: Option<f64>,
}

impl InputKling25TurboBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputKling25TurboAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKling25TurboResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKling25TurboStartImage) -> Self {
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

    /// Consumes the builder and constructs a [`InputKling25Turbo`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKling25TurboBuilder::prompt)
    /// - [`aspect_ratio`](InputKling25TurboBuilder::aspect_ratio)
    /// - [`duration_ms`](InputKling25TurboBuilder::duration_ms)
    pub fn build(self) -> Result<InputKling25Turbo, BuildError> {
        Ok(InputKling25Turbo {
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
