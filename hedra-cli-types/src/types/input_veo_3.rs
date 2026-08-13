pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `veo-3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution, start_image
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputVeo3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 20000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputVeo3AspectRatio,
    /// Output resolution.
    pub resolution: InputVeo3Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Start frame (image-to-video). At most 8 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputVeo3StartImage>,
    /// What to avoid in the generated video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Quality level to generate at.
    pub quality: InputVeo3Quality,
}

impl InputVeo3 {
    pub fn builder() -> InputVeo3Builder {
        <InputVeo3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputVeo3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputVeo3AspectRatio>,
    resolution: Option<InputVeo3Resolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    start_image: Option<InputVeo3StartImage>,
    negative_prompt: Option<String>,
    seed: Option<i64>,
    quality: Option<InputVeo3Quality>,
}

impl InputVeo3Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputVeo3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputVeo3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn generate_audio(mut self, value: bool) -> Self {
        self.generate_audio = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputVeo3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn quality(mut self, value: InputVeo3Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputVeo3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputVeo3Builder::prompt)
    /// - [`aspect_ratio`](InputVeo3Builder::aspect_ratio)
    /// - [`resolution`](InputVeo3Builder::resolution)
    /// - [`duration_ms`](InputVeo3Builder::duration_ms)
    /// - [`quality`](InputVeo3Builder::quality)
    pub fn build(self) -> Result<InputVeo3, BuildError> {
        Ok(InputVeo3 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            generate_audio: self.generate_audio,
            start_image: self.start_image,
            negative_prompt: self.negative_prompt,
            seed: self.seed,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
