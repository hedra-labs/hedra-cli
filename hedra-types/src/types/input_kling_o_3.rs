pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-o3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, images, prompt; accepts quality: standard; resolution: 720p
/// (2) requires: aspect_ratio, duration_ms, images, prompt, resolution; accepts quality: pro; resolution: 1080p | 4K
/// (3) requires: aspect_ratio, duration_ms, prompt; must omit: end_image, images, start_image; accepts quality: standard; resolution: 720p
/// (4) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: end_image, images, start_image; accepts quality: pro; resolution: 1080p | 4K
/// (5) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; accepts quality: pro; resolution: 1080p | 4K
/// (6) requires: aspect_ratio, duration_ms, prompt, start_image; accepts quality: standard; resolution: 720p
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingO3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputKlingO3AspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKlingO3Resolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputKlingO3StartImage>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputKlingO3EndImage>,
    /// Reference images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputKlingO3ImagesItem>>,
    /// Quality level to generate at.
    pub quality: InputKlingO3Quality,
}

impl InputKlingO3 {
    pub fn builder() -> InputKlingO3Builder {
        <InputKlingO3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingO3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    generate_audio: Option<bool>,
    aspect_ratio: Option<InputKlingO3AspectRatio>,
    resolution: Option<InputKlingO3Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputKlingO3StartImage>,
    end_image: Option<InputKlingO3EndImage>,
    images: Option<Vec<InputKlingO3ImagesItem>>,
    quality: Option<InputKlingO3Quality>,
}

impl InputKlingO3Builder {
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

    pub fn aspect_ratio(mut self, value: InputKlingO3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingO3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKlingO3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputKlingO3EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputKlingO3ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn quality(mut self, value: InputKlingO3Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingO3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKlingO3Builder::prompt)
    /// - [`aspect_ratio`](InputKlingO3Builder::aspect_ratio)
    /// - [`duration_ms`](InputKlingO3Builder::duration_ms)
    /// - [`quality`](InputKlingO3Builder::quality)
    pub fn build(self) -> Result<InputKlingO3, BuildError> {
        Ok(InputKlingO3 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            generate_audio: self.generate_audio,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            images: self.images,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
