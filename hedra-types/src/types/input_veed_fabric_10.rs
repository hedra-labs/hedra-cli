pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `veed-fabric-10`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputVeedFabric10 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputVeedFabric10AspectRatio,
    /// Output resolution.
    pub resolution: InputVeedFabric10Resolution,
    /// Start frame (image-to-video).
    pub start_image: InputVeedFabric10StartImage,
    /// Driving audio.
    pub audio: InputVeedFabric10Audio,
}

impl InputVeedFabric10 {
    pub fn builder() -> InputVeedFabric10Builder {
        <InputVeedFabric10Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputVeedFabric10Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputVeedFabric10AspectRatio>,
    resolution: Option<InputVeedFabric10Resolution>,
    start_image: Option<InputVeedFabric10StartImage>,
    audio: Option<InputVeedFabric10Audio>,
}

impl InputVeedFabric10Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputVeedFabric10AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputVeedFabric10Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputVeedFabric10StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputVeedFabric10Audio) -> Self {
        self.audio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputVeedFabric10`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputVeedFabric10Builder::prompt)
    /// - [`aspect_ratio`](InputVeedFabric10Builder::aspect_ratio)
    /// - [`resolution`](InputVeedFabric10Builder::resolution)
    /// - [`start_image`](InputVeedFabric10Builder::start_image)
    /// - [`audio`](InputVeedFabric10Builder::audio)
    pub fn build(self) -> Result<InputVeedFabric10, BuildError> {
        Ok(InputVeedFabric10 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
        })
    }
}
