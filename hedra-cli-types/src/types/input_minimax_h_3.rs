pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `minimax-h3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: duration_ms, prompt, resolution, start_image; must omit: aspect_ratio, audios, end_image, images, videos
/// (2) requires: duration_ms, end_image, prompt, resolution, start_image; must omit: aspect_ratio, audios, images, videos
/// (3) requires: aspect_ratio, duration_ms, images, prompt, resolution; must omit: end_image, start_image
/// (4) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: audios, end_image, images, start_image, videos; accepts aspect_ratio: 1:1 | 3:4 | 4:3 | 16:9 | 21:9 | 9:16
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMinimaxH3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. From 1 to 7000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    pub resolution: InputMinimaxH3Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputMinimaxH3StartImage>,
    /// End frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputMinimaxH3EndImage>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputMinimaxH3AspectRatio>,
    /// Reference images. 1 to 5 images, each from 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputMinimaxH3ImagesItem>>,
    /// Reference videos. 1 to 3 videos, each from 2s to 15s and at most 524.2 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputMinimaxH3VideosItem>>,
    /// Reference audios. 1 to 3 audio files, each from 2s to 15s and at most 104.8 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audios: Option<Vec<InputMinimaxH3AudiosItem>>,
}

impl InputMinimaxH3 {
    pub fn builder() -> InputMinimaxH3Builder {
        <InputMinimaxH3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMinimaxH3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputMinimaxH3Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputMinimaxH3StartImage>,
    end_image: Option<InputMinimaxH3EndImage>,
    aspect_ratio: Option<InputMinimaxH3AspectRatio>,
    images: Option<Vec<InputMinimaxH3ImagesItem>>,
    videos: Option<Vec<InputMinimaxH3VideosItem>>,
    audios: Option<Vec<InputMinimaxH3AudiosItem>>,
}

impl InputMinimaxH3Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputMinimaxH3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputMinimaxH3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputMinimaxH3EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputMinimaxH3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputMinimaxH3ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputMinimaxH3VideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    pub fn audios(mut self, value: Vec<InputMinimaxH3AudiosItem>) -> Self {
        self.audios = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMinimaxH3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputMinimaxH3Builder::prompt)
    /// - [`resolution`](InputMinimaxH3Builder::resolution)
    /// - [`duration_ms`](InputMinimaxH3Builder::duration_ms)
    pub fn build(self) -> Result<InputMinimaxH3, BuildError> {
        Ok(InputMinimaxH3 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            aspect_ratio: self.aspect_ratio,
            images: self.images,
            videos: self.videos,
            audios: self.audios,
        })
    }
}
