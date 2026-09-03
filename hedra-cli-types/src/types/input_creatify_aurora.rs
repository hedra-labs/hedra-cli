pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `creatify-aurora`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputCreatifyAurora {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Output resolution.
    pub resolution: InputCreatifyAuroraResolution,
    /// Start frame. At most 10.4 MB.
    pub start_image: InputCreatifyAuroraStartImage,
    /// Driving audio. At most 104.8 MB.
    pub audio: InputCreatifyAuroraAudio,
}

impl InputCreatifyAurora {
    pub fn builder() -> InputCreatifyAuroraBuilder {
        <InputCreatifyAuroraBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputCreatifyAuroraBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputCreatifyAuroraResolution>,
    start_image: Option<InputCreatifyAuroraStartImage>,
    audio: Option<InputCreatifyAuroraAudio>,
}

impl InputCreatifyAuroraBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputCreatifyAuroraResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputCreatifyAuroraStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputCreatifyAuroraAudio) -> Self {
        self.audio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputCreatifyAurora`].
    /// This method will fail if any of the following fields are not set:
    /// - [`resolution`](InputCreatifyAuroraBuilder::resolution)
    /// - [`start_image`](InputCreatifyAuroraBuilder::start_image)
    /// - [`audio`](InputCreatifyAuroraBuilder::audio)
    pub fn build(self) -> Result<InputCreatifyAurora, BuildError> {
        Ok(InputCreatifyAurora {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
        })
    }
}
