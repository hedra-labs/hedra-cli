pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `omnihuman-20`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputOmnihuman20 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputOmnihuman20Resolution>,
    /// Start frame. At most 10.4 MB.
    pub start_image: InputOmnihuman20StartImage,
    /// Driving audio. From 4s to 300s and at most 104.8 MB.
    pub audio: InputOmnihuman20Audio,
}

impl InputOmnihuman20 {
    pub fn builder() -> InputOmnihuman20Builder {
        <InputOmnihuman20Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputOmnihuman20Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputOmnihuman20Resolution>,
    start_image: Option<InputOmnihuman20StartImage>,
    audio: Option<InputOmnihuman20Audio>,
}

impl InputOmnihuman20Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputOmnihuman20Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputOmnihuman20StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputOmnihuman20Audio) -> Self {
        self.audio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputOmnihuman20`].
    /// This method will fail if any of the following fields are not set:
    /// - [`start_image`](InputOmnihuman20Builder::start_image)
    /// - [`audio`](InputOmnihuman20Builder::audio)
    pub fn build(self) -> Result<InputOmnihuman20, BuildError> {
        Ok(InputOmnihuman20 {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            resolution: self.resolution,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
        })
    }
}
