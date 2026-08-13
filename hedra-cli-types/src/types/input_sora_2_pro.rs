pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `sora-2-pro`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution, start_image
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSora2Pro {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 5000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputSora2ProAspectRatio,
    /// Output resolution.
    pub resolution: InputSora2ProResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video). At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputSora2ProStartImage>,
}

impl InputSora2Pro {
    pub fn builder() -> InputSora2ProBuilder {
        <InputSora2ProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputSora2ProBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputSora2ProAspectRatio>,
    resolution: Option<InputSora2ProResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputSora2ProStartImage>,
}

impl InputSora2ProBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputSora2ProAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputSora2ProResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputSora2ProStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputSora2Pro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputSora2ProBuilder::prompt)
    /// - [`aspect_ratio`](InputSora2ProBuilder::aspect_ratio)
    /// - [`resolution`](InputSora2ProBuilder::resolution)
    /// - [`duration_ms`](InputSora2ProBuilder::duration_ms)
    pub fn build(self) -> Result<InputSora2Pro, BuildError> {
        Ok(InputSora2Pro {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
        })
    }
}
