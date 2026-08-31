pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `seedance-20`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution; accepts quality: standard
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution; accepts quality: fast; resolution: 480p | 720p
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSeedance20 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputSeedance20AspectRatio,
    /// Output resolution.
    pub resolution: InputSeedance20Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Start frame. From 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputSeedance20StartImage>,
    /// End frame. From 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputSeedance20EndImage>,
    /// Reference images. 1 to 9 images, each from 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputSeedance20ImagesItem>>,
    /// Reference videos. 1 to 3 videos, each from 2s to 15s and at most 524.2 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputSeedance20VideosItem>>,
    /// Reference audios. 1 to 3 audio files, each at most 104.8 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audios: Option<Vec<InputSeedance20AudiosItem>>,
    /// Quality level to generate at. `standard` — the full model, and the only level that reaches 1080p and 4K. `fast` — tuned for turnaround, at 480p and 720p.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputSeedance20Quality>,
}

impl InputSeedance20 {
    pub fn builder() -> InputSeedance20Builder {
        <InputSeedance20Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputSeedance20Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputSeedance20AspectRatio>,
    resolution: Option<InputSeedance20Resolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    start_image: Option<InputSeedance20StartImage>,
    end_image: Option<InputSeedance20EndImage>,
    images: Option<Vec<InputSeedance20ImagesItem>>,
    videos: Option<Vec<InputSeedance20VideosItem>>,
    audios: Option<Vec<InputSeedance20AudiosItem>>,
    quality: Option<InputSeedance20Quality>,
}

impl InputSeedance20Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputSeedance20AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputSeedance20Resolution) -> Self {
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

    pub fn start_image(mut self, value: InputSeedance20StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputSeedance20EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputSeedance20ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputSeedance20VideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    pub fn audios(mut self, value: Vec<InputSeedance20AudiosItem>) -> Self {
        self.audios = Some(value);
        self
    }

    pub fn quality(mut self, value: InputSeedance20Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputSeedance20`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputSeedance20Builder::prompt)
    /// - [`aspect_ratio`](InputSeedance20Builder::aspect_ratio)
    /// - [`resolution`](InputSeedance20Builder::resolution)
    /// - [`duration_ms`](InputSeedance20Builder::duration_ms)
    pub fn build(self) -> Result<InputSeedance20, BuildError> {
        Ok(InputSeedance20 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            generate_audio: self.generate_audio,
            start_image: self.start_image,
            end_image: self.end_image,
            images: self.images,
            videos: self.videos,
            audios: self.audios,
            quality: self.quality,
        })
    }
}
