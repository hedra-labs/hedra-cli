pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux-3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, end_image, prompt, resolution, start_image
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: end_image, start_image
/// (3) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; must omit: end_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputFlux3AspectRatio,
    /// Output resolution.
    pub resolution: InputFlux3Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputFlux3StartImage>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputFlux3EndImage>,
}

impl InputFlux3 {
    pub fn builder() -> InputFlux3Builder {
        <InputFlux3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputFlux3AspectRatio>,
    resolution: Option<InputFlux3Resolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    start_image: Option<InputFlux3StartImage>,
    end_image: Option<InputFlux3EndImage>,
}

impl InputFlux3Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputFlux3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputFlux3Resolution) -> Self {
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

    pub fn start_image(mut self, value: InputFlux3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputFlux3EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFlux3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFlux3Builder::prompt)
    /// - [`aspect_ratio`](InputFlux3Builder::aspect_ratio)
    /// - [`resolution`](InputFlux3Builder::resolution)
    /// - [`duration_ms`](InputFlux3Builder::duration_ms)
    pub fn build(self) -> Result<InputFlux3, BuildError> {
        Ok(InputFlux3 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            generate_audio: self.generate_audio,
            start_image: self.start_image,
            end_image: self.end_image,
        })
    }
}
