pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `heygen-photo-avatar-4`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputHeygenPhotoAvatar4 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Output aspect ratio.
    pub aspect_ratio: InputHeygenPhotoAvatar4AspectRatio,
    /// Output resolution.
    pub resolution: InputHeygenPhotoAvatar4Resolution,
    /// Start frame. At most 10.4 MB.
    pub start_image: InputHeygenPhotoAvatar4StartImage,
    /// Driving audio. At most 104.8 MB.
    pub audio: InputHeygenPhotoAvatar4Audio,
}

impl InputHeygenPhotoAvatar4 {
    pub fn builder() -> InputHeygenPhotoAvatar4Builder {
        <InputHeygenPhotoAvatar4Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputHeygenPhotoAvatar4Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputHeygenPhotoAvatar4AspectRatio>,
    resolution: Option<InputHeygenPhotoAvatar4Resolution>,
    start_image: Option<InputHeygenPhotoAvatar4StartImage>,
    audio: Option<InputHeygenPhotoAvatar4Audio>,
}

impl InputHeygenPhotoAvatar4Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputHeygenPhotoAvatar4AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputHeygenPhotoAvatar4Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputHeygenPhotoAvatar4StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputHeygenPhotoAvatar4Audio) -> Self {
        self.audio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputHeygenPhotoAvatar4`].
    /// This method will fail if any of the following fields are not set:
    /// - [`aspect_ratio`](InputHeygenPhotoAvatar4Builder::aspect_ratio)
    /// - [`resolution`](InputHeygenPhotoAvatar4Builder::resolution)
    /// - [`start_image`](InputHeygenPhotoAvatar4Builder::start_image)
    /// - [`audio`](InputHeygenPhotoAvatar4Builder::audio)
    pub fn build(self) -> Result<InputHeygenPhotoAvatar4, BuildError> {
        Ok(InputHeygenPhotoAvatar4 {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
        })
    }
}
