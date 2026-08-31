pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `gemini-omni-flash`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGeminiOmniFlash {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputGeminiOmniFlashAspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputGeminiOmniFlashResolution>,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Start frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputGeminiOmniFlashStartImage>,
    /// Source video. From 1s to 10s and at most 524.2 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_video: Option<InputGeminiOmniFlashSourceVideo>,
    /// Reference images. 1 to 10 images, each at most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGeminiOmniFlashImagesItem>>,
    /// Reference videos. 1 to 3 videos, each at most 10s and at most 524.2 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputGeminiOmniFlashVideosItem>>,
}

impl InputGeminiOmniFlash {
    pub fn builder() -> InputGeminiOmniFlashBuilder {
        <InputGeminiOmniFlashBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGeminiOmniFlashBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputGeminiOmniFlashAspectRatio>,
    resolution: Option<InputGeminiOmniFlashResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputGeminiOmniFlashStartImage>,
    source_video: Option<InputGeminiOmniFlashSourceVideo>,
    images: Option<Vec<InputGeminiOmniFlashImagesItem>>,
    videos: Option<Vec<InputGeminiOmniFlashVideosItem>>,
}

impl InputGeminiOmniFlashBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputGeminiOmniFlashAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGeminiOmniFlashResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputGeminiOmniFlashStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputGeminiOmniFlashSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputGeminiOmniFlashImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputGeminiOmniFlashVideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGeminiOmniFlash`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGeminiOmniFlashBuilder::prompt)
    /// - [`aspect_ratio`](InputGeminiOmniFlashBuilder::aspect_ratio)
    pub fn build(self) -> Result<InputGeminiOmniFlash, BuildError> {
        Ok(InputGeminiOmniFlash {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms,
            start_image: self.start_image,
            source_video: self.source_video,
            images: self.images,
            videos: self.videos,
        })
    }
}
