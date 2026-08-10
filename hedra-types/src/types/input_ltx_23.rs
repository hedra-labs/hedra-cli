pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `ltx-2-3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; accepts quality: fast
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: end_image, start_image; accepts aspect_ratio: 16:9 | 9:16; quality: fast
/// (3) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; accepts duration_ms: 6000 | 8000 | 10000; quality: pro
/// (4) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: end_image, start_image; accepts aspect_ratio: 16:9 | 9:16; duration_ms: 6000 | 8000 | 10000; quality: pro
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputLtx23 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    pub resolution: InputLtx23Resolution,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Output aspect ratio.
    pub aspect_ratio: InputLtx23AspectRatio,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputLtx23StartImage>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputLtx23EndImage>,
    /// Quality level to generate at.
    pub quality: InputLtx23Quality,
}

impl InputLtx23 {
    pub fn builder() -> InputLtx23Builder {
        <InputLtx23Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputLtx23Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputLtx23Resolution>,
    generate_audio: Option<bool>,
    duration_ms: Option<i64>,
    aspect_ratio: Option<InputLtx23AspectRatio>,
    start_image: Option<InputLtx23StartImage>,
    end_image: Option<InputLtx23EndImage>,
    quality: Option<InputLtx23Quality>,
}

impl InputLtx23Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputLtx23Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn generate_audio(mut self, value: bool) -> Self {
        self.generate_audio = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputLtx23AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputLtx23StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputLtx23EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn quality(mut self, value: InputLtx23Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputLtx23`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputLtx23Builder::prompt)
    /// - [`resolution`](InputLtx23Builder::resolution)
    /// - [`duration_ms`](InputLtx23Builder::duration_ms)
    /// - [`aspect_ratio`](InputLtx23Builder::aspect_ratio)
    /// - [`quality`](InputLtx23Builder::quality)
    pub fn build(self) -> Result<InputLtx23, BuildError> {
        Ok(InputLtx23 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            generate_audio: self.generate_audio,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
