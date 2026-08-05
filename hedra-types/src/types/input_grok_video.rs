pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `grok-video`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: start_image; accepts aspect_ratio: 1:1 | 4:3 | 3:4 | 16:9 | 9:16 | 3:2 | 2:3
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution, start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGrokVideo {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputGrokVideoAspectRatio,
    /// Output resolution.
    pub resolution: InputGrokVideoResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputGrokVideoStartImage>,
}

impl InputGrokVideo {
    pub fn builder() -> InputGrokVideoBuilder {
        <InputGrokVideoBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGrokVideoBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputGrokVideoAspectRatio>,
    resolution: Option<InputGrokVideoResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputGrokVideoStartImage>,
}

impl InputGrokVideoBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputGrokVideoAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGrokVideoResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputGrokVideoStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGrokVideo`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGrokVideoBuilder::prompt)
    /// - [`aspect_ratio`](InputGrokVideoBuilder::aspect_ratio)
    /// - [`resolution`](InputGrokVideoBuilder::resolution)
    /// - [`duration_ms`](InputGrokVideoBuilder::duration_ms)
    pub fn build(self) -> Result<InputGrokVideo, BuildError> {
        Ok(InputGrokVideo {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
        })
    }
}
