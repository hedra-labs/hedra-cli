pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-v3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; must omit: end_image; accepts quality: pro; resolution: 1080p | 4K
/// (2) requires: aspect_ratio, duration_ms, end_image, prompt, resolution, start_image; accepts quality: pro; resolution: 1080p | 4K
/// (3) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: end_image, start_image; accepts quality: pro; resolution: 1080p | 4K
/// (4) requires: aspect_ratio, duration_ms, prompt, start_image; must omit: end_image; accepts quality: standard; resolution: 720p
/// (5) requires: aspect_ratio, duration_ms, end_image, prompt, start_image; accepts quality: standard; resolution: 720p
/// (6) requires: aspect_ratio, duration_ms, prompt; must omit: end_image, start_image; accepts quality: standard; resolution: 720p
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingV3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// What to avoid in the generated video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Output aspect ratio.
    pub aspect_ratio: InputKlingV3AspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKlingV3Resolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputKlingV3StartImage>,
    /// How closely the model follows the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub cfg_scale: Option<f64>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputKlingV3EndImage>,
    /// Quality level to generate at.
    pub quality: InputKlingV3Quality,
}

impl InputKlingV3 {
    pub fn builder() -> InputKlingV3Builder {
        <InputKlingV3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingV3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    generate_audio: Option<bool>,
    negative_prompt: Option<String>,
    aspect_ratio: Option<InputKlingV3AspectRatio>,
    resolution: Option<InputKlingV3Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputKlingV3StartImage>,
    cfg_scale: Option<f64>,
    end_image: Option<InputKlingV3EndImage>,
    quality: Option<InputKlingV3Quality>,
}

impl InputKlingV3Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn generate_audio(mut self, value: bool) -> Self {
        self.generate_audio = Some(value);
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputKlingV3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingV3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKlingV3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn cfg_scale(mut self, value: f64) -> Self {
        self.cfg_scale = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputKlingV3EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn quality(mut self, value: InputKlingV3Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingV3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKlingV3Builder::prompt)
    /// - [`aspect_ratio`](InputKlingV3Builder::aspect_ratio)
    /// - [`duration_ms`](InputKlingV3Builder::duration_ms)
    /// - [`quality`](InputKlingV3Builder::quality)
    pub fn build(self) -> Result<InputKlingV3, BuildError> {
        Ok(InputKlingV3 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            generate_audio: self.generate_audio,
            negative_prompt: self.negative_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            cfg_scale: self.cfg_scale,
            end_image: self.end_image,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
