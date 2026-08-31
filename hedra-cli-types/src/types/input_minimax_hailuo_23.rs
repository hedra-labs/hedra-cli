pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `minimax-hailuo-23`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: prompt, start_image; must omit: aspect_ratio; accepts duration_ms: 6000; quality: pro | fast-pro; resolution: 1080p
/// (2) requires: duration_ms, prompt, start_image; must omit: aspect_ratio; accepts quality: standard | fast-standard; resolution: 768p
/// (3) requires: prompt; must omit: start_image; accepts duration_ms: 6000; quality: pro; resolution: 1080p
/// (4) requires: duration_ms, prompt; must omit: start_image; accepts quality: standard; resolution: 768p
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputMinimaxHailuo23 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputMinimaxHailuo23Resolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame. The output video follows this image's aspect ratio. At most 20 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputMinimaxHailuo23StartImage>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputMinimaxHailuo23AspectRatio>,
    /// Quality level to generate at. `standard` — 768p, for everyday motion. `pro` — 1080p, with smoother motion and sharper detail. `fast-standard` — 768p on the low-latency path, from a start frame only. `fast-pro` — 1080p on the low-latency path, from a start frame only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputMinimaxHailuo23Quality>,
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
    resolution: Option<InputMinimaxHailuo23Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputMinimaxHailuo23StartImage>,
    aspect_ratio: Option<InputMinimaxHailuo23AspectRatio>,
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

    pub fn aspect_ratio(mut self, value: InputMinimaxHailuo23AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
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
    pub fn build(self) -> Result<InputMinimaxHailuo23, BuildError> {
        Ok(InputMinimaxHailuo23 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            aspect_ratio: self.aspect_ratio,
            quality: self.quality,
        })
    }
}
