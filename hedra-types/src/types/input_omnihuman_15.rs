pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `omnihuman-15`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputOmnihuman15 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputOmnihuman15AspectRatio>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputOmnihuman15Resolution>,
    /// Start frame (image-to-video).
    pub start_image: InputOmnihuman15StartImage,
    /// Driving audio.
    pub audio: InputOmnihuman15Audio,
}

impl InputOmnihuman15 {
    pub fn builder() -> InputOmnihuman15Builder {
        <InputOmnihuman15Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputOmnihuman15Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputOmnihuman15AspectRatio>,
    resolution: Option<InputOmnihuman15Resolution>,
    start_image: Option<InputOmnihuman15StartImage>,
    audio: Option<InputOmnihuman15Audio>,
}

impl InputOmnihuman15Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputOmnihuman15AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputOmnihuman15Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputOmnihuman15StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputOmnihuman15Audio) -> Self {
        self.audio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputOmnihuman15`].
    /// This method will fail if any of the following fields are not set:
    /// - [`start_image`](InputOmnihuman15Builder::start_image)
    /// - [`audio`](InputOmnihuman15Builder::audio)
    pub fn build(self) -> Result<InputOmnihuman15, BuildError> {
        Ok(InputOmnihuman15 {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
        })
    }
}
