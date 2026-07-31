pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `veo-31`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputVeo31 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputVeo31AspectRatio,
    /// Output resolution.
    pub resolution: InputVeo31Resolution,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// What to avoid in the generated video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputVeo31StartImage>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputVeo31EndImage>,
    /// Source video (video-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_video: Option<InputVeo31SourceVideo>,
    /// Reference images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputVeo31ImagesItem>>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Quality level to generate at.
    pub quality: InputVeo31Quality,
}

impl InputVeo31 {
    pub fn builder() -> InputVeo31Builder {
        <InputVeo31Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputVeo31Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputVeo31AspectRatio>,
    resolution: Option<InputVeo31Resolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    negative_prompt: Option<String>,
    start_image: Option<InputVeo31StartImage>,
    end_image: Option<InputVeo31EndImage>,
    source_video: Option<InputVeo31SourceVideo>,
    images: Option<Vec<InputVeo31ImagesItem>>,
    seed: Option<i64>,
    quality: Option<InputVeo31Quality>,
}

impl InputVeo31Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputVeo31AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputVeo31Resolution) -> Self {
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

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn start_image(mut self, value: InputVeo31StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputVeo31EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputVeo31SourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputVeo31ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn quality(mut self, value: InputVeo31Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputVeo31`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputVeo31Builder::prompt)
    /// - [`aspect_ratio`](InputVeo31Builder::aspect_ratio)
    /// - [`resolution`](InputVeo31Builder::resolution)
    /// - [`quality`](InputVeo31Builder::quality)
    pub fn build(self) -> Result<InputVeo31, BuildError> {
        Ok(InputVeo31 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms,
            generate_audio: self.generate_audio,
            negative_prompt: self.negative_prompt,
            start_image: self.start_image,
            end_image: self.end_image,
            source_video: self.source_video,
            images: self.images,
            seed: self.seed,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
