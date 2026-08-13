pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `vidu-q3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: duration_ms, prompt, resolution, start_image; must omit: aspect_ratio, end_image
/// (2) requires: duration_ms, end_image, prompt, resolution, start_image; must omit: aspect_ratio
/// (3) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: end_image, start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputViduQ3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Output resolution.
    pub resolution: InputViduQ3Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video). At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputViduQ3StartImage>,
    /// End frame (first-last-frame-to-video). At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputViduQ3EndImage>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputViduQ3AspectRatio>,
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
    resolution: Option<InputViduQ3Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputViduQ3StartImage>,
    end_image: Option<InputViduQ3EndImage>,
    aspect_ratio: Option<InputViduQ3AspectRatio>,
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

    pub fn aspect_ratio(mut self, value: InputViduQ3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
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
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            aspect_ratio: self.aspect_ratio,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
