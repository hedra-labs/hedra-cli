pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `minimax-hailuo-23`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMinimaxHailuo23 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputMinimaxHailuo23AspectRatio>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputMinimaxHailuo23Resolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputMinimaxHailuo23StartImage>,
    /// Quality level to generate at.
    pub quality: InputMinimaxHailuo23Quality,
}

impl InputMinimaxHailuo23 {
    pub fn builder() -> InputMinimaxHailuo23Builder {
        <InputMinimaxHailuo23Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMinimaxHailuo23Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputMinimaxHailuo23AspectRatio>,
    resolution: Option<InputMinimaxHailuo23Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputMinimaxHailuo23StartImage>,
    quality: Option<InputMinimaxHailuo23Quality>,
}

impl InputMinimaxHailuo23Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputMinimaxHailuo23AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputMinimaxHailuo23Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputMinimaxHailuo23StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn quality(mut self, value: InputMinimaxHailuo23Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMinimaxHailuo23`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputMinimaxHailuo23Builder::prompt)
    /// - [`duration_ms`](InputMinimaxHailuo23Builder::duration_ms)
    /// - [`quality`](InputMinimaxHailuo23Builder::quality)
    pub fn build(self) -> Result<InputMinimaxHailuo23, BuildError> {
        Ok(InputMinimaxHailuo23 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
