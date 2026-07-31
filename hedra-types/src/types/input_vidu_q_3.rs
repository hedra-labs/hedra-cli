pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `vidu-q3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution
/// (2) requires: duration_ms, end_image, prompt, resolution, start_image
/// (3) requires: duration_ms, prompt, resolution, start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputViduQ3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputViduQ3AspectRatio>,
    /// Output resolution.
    pub resolution: InputViduQ3Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputViduQ3StartImage>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputViduQ3EndImage>,
    /// Quality level to generate at.
    pub quality: InputViduQ3Quality,
}

impl InputViduQ3 {
    pub fn builder() -> InputViduQ3Builder {
        <InputViduQ3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputViduQ3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    seed: Option<i64>,
    aspect_ratio: Option<InputViduQ3AspectRatio>,
    resolution: Option<InputViduQ3Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputViduQ3StartImage>,
    end_image: Option<InputViduQ3EndImage>,
    quality: Option<InputViduQ3Quality>,
}

impl InputViduQ3Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputViduQ3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputViduQ3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputViduQ3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputViduQ3EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn quality(mut self, value: InputViduQ3Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputViduQ3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputViduQ3Builder::prompt)
    /// - [`resolution`](InputViduQ3Builder::resolution)
    /// - [`duration_ms`](InputViduQ3Builder::duration_ms)
    /// - [`quality`](InputViduQ3Builder::quality)
    pub fn build(self) -> Result<InputViduQ3, BuildError> {
        Ok(InputViduQ3 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            seed: self.seed,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
