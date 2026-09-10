pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `gemini-omni-flash-11`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGeminiOmniFlash11 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputGeminiOmniFlash11AspectRatio,
    /// Output resolution.
    pub resolution: InputGeminiOmniFlash11Resolution,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Start frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputGeminiOmniFlash11StartImage>,
    /// End frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputGeminiOmniFlash11EndImage>,
    /// Source video. From 1s to 10s and at most 524.2 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_video: Option<InputGeminiOmniFlash11SourceVideo>,
    /// Reference images. 1 to 10 images, each at most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGeminiOmniFlash11ImagesItem>>,
    /// Reference videos. 1 to 3 videos, each at most 10s and at most 524.2 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<InputGeminiOmniFlash11VideosItem>>,
}

impl InputGeminiOmniFlash11 {
    pub fn builder() -> InputGeminiOmniFlash11Builder {
        <InputGeminiOmniFlash11Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGeminiOmniFlash11Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputGeminiOmniFlash11AspectRatio>,
    resolution: Option<InputGeminiOmniFlash11Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputGeminiOmniFlash11StartImage>,
    end_image: Option<InputGeminiOmniFlash11EndImage>,
    source_video: Option<InputGeminiOmniFlash11SourceVideo>,
    images: Option<Vec<InputGeminiOmniFlash11ImagesItem>>,
    videos: Option<Vec<InputGeminiOmniFlash11VideosItem>>,
}

impl InputGeminiOmniFlash11Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputGeminiOmniFlash11AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGeminiOmniFlash11Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputGeminiOmniFlash11StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputGeminiOmniFlash11EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputGeminiOmniFlash11SourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputGeminiOmniFlash11ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn videos(mut self, value: Vec<InputGeminiOmniFlash11VideosItem>) -> Self {
        self.videos = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGeminiOmniFlash11`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGeminiOmniFlash11Builder::prompt)
    /// - [`aspect_ratio`](InputGeminiOmniFlash11Builder::aspect_ratio)
    /// - [`resolution`](InputGeminiOmniFlash11Builder::resolution)
    pub fn build(self) -> Result<InputGeminiOmniFlash11, BuildError> {
        Ok(InputGeminiOmniFlash11 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms,
            start_image: self.start_image,
            end_image: self.end_image,
            source_video: self.source_video,
            images: self.images,
            videos: self.videos,
        })
    }
}
