pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `wan-3-0`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; must omit: audios, images, videos
/// (2) requires: aspect_ratio, duration_ms, images, prompt, resolution; must omit: end_image, start_image
/// (3) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: audios, end_image, images, start_image, videos
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputWan30 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 5000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputWan30AspectRatio,
    /// Output resolution.
    pub resolution: InputWan30Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Seed for reproducible output; omit for a random seed. From 0 to 2147483647.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Start frame. At most 20 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputWan30StartImage>,
    /// End frame. At most 20 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputWan30EndImage>,
    /// Reference images. 1 to 10 images, each at most 20 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputWan30ImagesItem>>,
    /// Reference videos. 1 to 5 videos, each at most 15s and at most 524.2 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputWan30VideosItem>>,
    /// Reference audios. 1 to 5 audio files, each at most 15s and at most 104.8 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audios: Option<Vec<InputWan30AudiosItem>>,
    /// Quality level to generate at. `standard` — the base tier. `prime` — Wan's higher-fidelity tier over the same options, for final output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputWan30Quality>,
}

impl InputWan30 {
    pub fn builder() -> InputWan30Builder {
        <InputWan30Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputWan30Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputWan30AspectRatio>,
    resolution: Option<InputWan30Resolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    seed: Option<i64>,
    start_image: Option<InputWan30StartImage>,
    end_image: Option<InputWan30EndImage>,
    images: Option<Vec<InputWan30ImagesItem>>,
    videos: Option<Vec<InputWan30VideosItem>>,
    audios: Option<Vec<InputWan30AudiosItem>>,
    quality: Option<InputWan30Quality>,
}

impl InputWan30Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputWan30AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputWan30Resolution) -> Self {
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

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputWan30StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputWan30EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputWan30ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputWan30VideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    pub fn audios(mut self, value: Vec<InputWan30AudiosItem>) -> Self {
        self.audios = Some(value);
        self
    }

    pub fn quality(mut self, value: InputWan30Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputWan30`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputWan30Builder::prompt)
    /// - [`aspect_ratio`](InputWan30Builder::aspect_ratio)
    /// - [`resolution`](InputWan30Builder::resolution)
    /// - [`duration_ms`](InputWan30Builder::duration_ms)
    pub fn build(self) -> Result<InputWan30, BuildError> {
        Ok(InputWan30 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            generate_audio: self.generate_audio,
            seed: self.seed,
            start_image: self.start_image,
            end_image: self.end_image,
            images: self.images,
            videos: self.videos,
            audios: self.audios,
            quality: self.quality,
        })
    }
}
