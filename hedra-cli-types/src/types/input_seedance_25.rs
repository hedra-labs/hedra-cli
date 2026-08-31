pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `seedance-25`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSeedance25 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputSeedance25AspectRatio,
    /// Output resolution.
    pub resolution: InputSeedance25Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Start frame. From 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputSeedance25StartImage>,
    /// End frame. From 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputSeedance25EndImage>,
    /// Reference images. 1 to 30 images, each from 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputSeedance25ImagesItem>>,
    /// Reference videos. 1 to 10 videos, each from 2s to 30s and at most 524.2 MB, at most 30s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputSeedance25VideosItem>>,
    /// Reference audios. 1 to 10 audio files, each at most 104.8 MB, at most 30s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audios: Option<Vec<InputSeedance25AudiosItem>>,
}

impl InputSeedance25 {
    pub fn builder() -> InputSeedance25Builder {
        <InputSeedance25Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputSeedance25Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputSeedance25AspectRatio>,
    resolution: Option<InputSeedance25Resolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    start_image: Option<InputSeedance25StartImage>,
    end_image: Option<InputSeedance25EndImage>,
    images: Option<Vec<InputSeedance25ImagesItem>>,
    videos: Option<Vec<InputSeedance25VideosItem>>,
    audios: Option<Vec<InputSeedance25AudiosItem>>,
}

impl InputSeedance25Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputSeedance25AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputSeedance25Resolution) -> Self {
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

    pub fn start_image(mut self, value: InputSeedance25StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputSeedance25EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputSeedance25ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputSeedance25VideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    pub fn audios(mut self, value: Vec<InputSeedance25AudiosItem>) -> Self {
        self.audios = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputSeedance25`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputSeedance25Builder::prompt)
    /// - [`aspect_ratio`](InputSeedance25Builder::aspect_ratio)
    /// - [`resolution`](InputSeedance25Builder::resolution)
    /// - [`duration_ms`](InputSeedance25Builder::duration_ms)
    pub fn build(self) -> Result<InputSeedance25, BuildError> {
        Ok(InputSeedance25 {
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
        })
    }
}
