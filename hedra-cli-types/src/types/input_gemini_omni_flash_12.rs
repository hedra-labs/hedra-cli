pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `gemini-omni-flash-12`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGeminiOmniFlash12 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputGeminiOmniFlash12AspectRatio,
    /// Output resolution.
    pub resolution: InputGeminiOmniFlash12Resolution,
    /// Start frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputGeminiOmniFlash12StartImage>,
    /// End frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputGeminiOmniFlash12EndImage>,
    /// Source video. From 1s to 15.083s and at most 524.2 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_video: Option<InputGeminiOmniFlash12SourceVideo>,
    /// Reference images. 1 to 10 images, each at most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGeminiOmniFlash12ImagesItem>>,
    /// Reference videos. 1 to 3 videos, each at most 15.15s and at most 524.2 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputGeminiOmniFlash12VideosItem>>,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

impl InputGeminiOmniFlash12 {
    pub fn builder() -> InputGeminiOmniFlash12Builder {
        <InputGeminiOmniFlash12Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGeminiOmniFlash12Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputGeminiOmniFlash12AspectRatio>,
    resolution: Option<InputGeminiOmniFlash12Resolution>,
    start_image: Option<InputGeminiOmniFlash12StartImage>,
    end_image: Option<InputGeminiOmniFlash12EndImage>,
    source_video: Option<InputGeminiOmniFlash12SourceVideo>,
    images: Option<Vec<InputGeminiOmniFlash12ImagesItem>>,
    videos: Option<Vec<InputGeminiOmniFlash12VideosItem>>,
    duration_ms: Option<i64>,
}

impl InputGeminiOmniFlash12Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputGeminiOmniFlash12AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGeminiOmniFlash12Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputGeminiOmniFlash12StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputGeminiOmniFlash12EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputGeminiOmniFlash12SourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputGeminiOmniFlash12ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputGeminiOmniFlash12VideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGeminiOmniFlash12`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGeminiOmniFlash12Builder::prompt)
    /// - [`aspect_ratio`](InputGeminiOmniFlash12Builder::aspect_ratio)
    /// - [`resolution`](InputGeminiOmniFlash12Builder::resolution)
    pub fn build(self) -> Result<InputGeminiOmniFlash12, BuildError> {
        Ok(InputGeminiOmniFlash12 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            source_video: self.source_video,
            images: self.images,
            videos: self.videos,
            duration_ms: self.duration_ms,
        })
    }
}
