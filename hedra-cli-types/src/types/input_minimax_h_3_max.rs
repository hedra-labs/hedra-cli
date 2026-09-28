pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `minimax-h3-max`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: duration_ms, prompt, resolution, start_image; must omit: aspect_ratio, audios, end_image, images, videos
/// (2) requires: duration_ms, end_image, prompt, resolution, start_image; must omit: aspect_ratio, audios, images, videos
/// (3) requires: aspect_ratio, duration_ms, images, prompt, resolution; must omit: end_image, start_image
/// (4) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: audios, end_image, images, start_image, videos; accepts aspect_ratio: 1:1 | 3:4 | 4:3 | 16:9 | 21:9 | 9:16
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMinimaxH3Max {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. From 1 to 7000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    pub resolution: InputMinimaxH3MaxResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Start frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputMinimaxH3MaxStartImage>,
    /// End frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputMinimaxH3MaxEndImage>,
    /// Output aspect ratio. Omitted or `adaptive` uses the supported ratio nearest the first reference image, else the first reference video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputMinimaxH3MaxAspectRatio>,
    /// Reference images. 1 to 4 images, each from 256px to 1024px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputMinimaxH3MaxImagesItem>>,
    /// Reference videos. 1 to 3 videos, each from 2s to 15s and at most 524.2 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputMinimaxH3MaxVideosItem>>,
    /// Reference audios. 1 to 3 audio files, each from 2s to 15s and at most 104.8 MB, at most 15s in total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audios: Option<Vec<InputMinimaxH3MaxAudiosItem>>,
}

impl InputMinimaxH3Max {
    pub fn builder() -> InputMinimaxH3MaxBuilder {
        <InputMinimaxH3MaxBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMinimaxH3MaxBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputMinimaxH3MaxResolution>,
    duration_ms: Option<i64>,
    enhance_prompt: Option<bool>,
    start_image: Option<InputMinimaxH3MaxStartImage>,
    end_image: Option<InputMinimaxH3MaxEndImage>,
    aspect_ratio: Option<InputMinimaxH3MaxAspectRatio>,
    images: Option<Vec<InputMinimaxH3MaxImagesItem>>,
    videos: Option<Vec<InputMinimaxH3MaxVideosItem>>,
    audios: Option<Vec<InputMinimaxH3MaxAudiosItem>>,
}

impl InputMinimaxH3MaxBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputMinimaxH3MaxResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn enhance_prompt(mut self, value: bool) -> Self {
        self.enhance_prompt = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputMinimaxH3MaxStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputMinimaxH3MaxEndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputMinimaxH3MaxAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputMinimaxH3MaxImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputMinimaxH3MaxVideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    pub fn audios(mut self, value: Vec<InputMinimaxH3MaxAudiosItem>) -> Self {
        self.audios = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMinimaxH3Max`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputMinimaxH3MaxBuilder::prompt)
    /// - [`resolution`](InputMinimaxH3MaxBuilder::resolution)
    /// - [`duration_ms`](InputMinimaxH3MaxBuilder::duration_ms)
    pub fn build(self) -> Result<InputMinimaxH3Max, BuildError> {
        Ok(InputMinimaxH3Max {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            enhance_prompt: self.enhance_prompt,
            start_image: self.start_image,
            end_image: self.end_image,
            aspect_ratio: self.aspect_ratio,
            images: self.images,
            videos: self.videos,
            audios: self.audios,
        })
    }
}
