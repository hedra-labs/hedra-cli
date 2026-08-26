pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `seedance-20-mini`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSeedance20Mini {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputSeedance20MiniAspectRatio,
    /// Output resolution.
    pub resolution: InputSeedance20MiniResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video). From 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputSeedance20MiniStartImage>,
    /// End frame (first-last-frame-to-video). From 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputSeedance20MiniEndImage>,
    /// Reference images. 1 to 9 images, each from 300px to 6000px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputSeedance20MiniImagesItem>>,
    /// Reference videos. 1 to 3 videos, each from 2s to 15s and at most 524.2 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputSeedance20MiniVideosItem>>,
    /// Reference audios. 1 to 3 audio files, each at most 104.8 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audios: Option<Vec<InputSeedance20MiniAudiosItem>>,
}

impl InputSeedance20Mini {
    pub fn builder() -> InputSeedance20MiniBuilder {
        <InputSeedance20MiniBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputSeedance20MiniBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputSeedance20MiniAspectRatio>,
    resolution: Option<InputSeedance20MiniResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputSeedance20MiniStartImage>,
    end_image: Option<InputSeedance20MiniEndImage>,
    images: Option<Vec<InputSeedance20MiniImagesItem>>,
    videos: Option<Vec<InputSeedance20MiniVideosItem>>,
    audios: Option<Vec<InputSeedance20MiniAudiosItem>>,
}

impl InputSeedance20MiniBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputSeedance20MiniAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputSeedance20MiniResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputSeedance20MiniStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputSeedance20MiniEndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputSeedance20MiniImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputSeedance20MiniVideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    pub fn audios(mut self, value: Vec<InputSeedance20MiniAudiosItem>) -> Self {
        self.audios = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputSeedance20Mini`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputSeedance20MiniBuilder::prompt)
    /// - [`aspect_ratio`](InputSeedance20MiniBuilder::aspect_ratio)
    /// - [`resolution`](InputSeedance20MiniBuilder::resolution)
    /// - [`duration_ms`](InputSeedance20MiniBuilder::duration_ms)
    pub fn build(self) -> Result<InputSeedance20Mini, BuildError> {
        Ok(InputSeedance20Mini {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            images: self.images,
            videos: self.videos,
            audios: self.audios,
        })
    }
}
