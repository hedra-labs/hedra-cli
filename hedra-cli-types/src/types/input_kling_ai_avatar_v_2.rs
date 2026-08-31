pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-ai-avatar-v2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingAiAvatarV2 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Output aspect ratio.
    pub aspect_ratio: InputKlingAiAvatarV2AspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKlingAiAvatarV2Resolution>,
    /// Start frame. At most 10.4 MB.
    pub start_image: InputKlingAiAvatarV2StartImage,
    /// Driving audio. From 2s to 60s and at most 5 MB.
    pub audio: InputKlingAiAvatarV2Audio,
    /// Quality level to generate at. `standard` — the base tier. `pro` — sharper detail and steadier motion at the same resolution, at a higher rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputKlingAiAvatarV2Quality>,
}

impl InputKlingAiAvatarV2 {
    pub fn builder() -> InputKlingAiAvatarV2Builder {
        <InputKlingAiAvatarV2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingAiAvatarV2Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputKlingAiAvatarV2AspectRatio>,
    resolution: Option<InputKlingAiAvatarV2Resolution>,
    start_image: Option<InputKlingAiAvatarV2StartImage>,
    audio: Option<InputKlingAiAvatarV2Audio>,
    quality: Option<InputKlingAiAvatarV2Quality>,
}

impl InputKlingAiAvatarV2Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputKlingAiAvatarV2AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingAiAvatarV2Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKlingAiAvatarV2StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputKlingAiAvatarV2Audio) -> Self {
        self.audio = Some(value);
        self
    }

    pub fn quality(mut self, value: InputKlingAiAvatarV2Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingAiAvatarV2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`aspect_ratio`](InputKlingAiAvatarV2Builder::aspect_ratio)
    /// - [`start_image`](InputKlingAiAvatarV2Builder::start_image)
    /// - [`audio`](InputKlingAiAvatarV2Builder::audio)
    pub fn build(self) -> Result<InputKlingAiAvatarV2, BuildError> {
        Ok(InputKlingAiAvatarV2 {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
            quality: self.quality,
        })
    }
}
