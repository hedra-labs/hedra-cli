pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `minimax-h3-max-turbo`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: duration_ms, prompt, resolution, start_image; must omit: aspect_ratio, end_image
/// (2) requires: duration_ms, end_image, prompt, resolution, start_image; must omit: aspect_ratio
/// (3) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: end_image, start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMinimaxH3MaxTurbo {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. From 1 to 7000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    pub resolution: InputMinimaxH3MaxTurboResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputMinimaxH3MaxTurboStartImage>,
    /// End frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputMinimaxH3MaxTurboEndImage>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputMinimaxH3MaxTurboAspectRatio>,
}

impl InputMinimaxH3MaxTurbo {
    pub fn builder() -> InputMinimaxH3MaxTurboBuilder {
        <InputMinimaxH3MaxTurboBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMinimaxH3MaxTurboBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputMinimaxH3MaxTurboResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputMinimaxH3MaxTurboStartImage>,
    end_image: Option<InputMinimaxH3MaxTurboEndImage>,
    aspect_ratio: Option<InputMinimaxH3MaxTurboAspectRatio>,
}

impl InputMinimaxH3MaxTurboBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputMinimaxH3MaxTurboResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputMinimaxH3MaxTurboStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputMinimaxH3MaxTurboEndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputMinimaxH3MaxTurboAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMinimaxH3MaxTurbo`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputMinimaxH3MaxTurboBuilder::prompt)
    /// - [`resolution`](InputMinimaxH3MaxTurboBuilder::resolution)
    /// - [`duration_ms`](InputMinimaxH3MaxTurboBuilder::duration_ms)
    pub fn build(self) -> Result<InputMinimaxH3MaxTurbo, BuildError> {
        Ok(InputMinimaxH3MaxTurbo {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            aspect_ratio: self.aspect_ratio,
        })
    }
}
