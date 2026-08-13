pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `veo-2`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: prompt, start_image; must omit: negative_prompt, seed
/// (2) requires: prompt; must omit: start_image
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputVeo2 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputVeo2AspectRatio>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputVeo2Resolution>,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Start frame (image-to-video). At most 8 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputVeo2StartImage>,
    /// What to avoid in the generated video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputVeo2 {
    pub fn builder() -> InputVeo2Builder {
        <InputVeo2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputVeo2Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputVeo2AspectRatio>,
    resolution: Option<InputVeo2Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputVeo2StartImage>,
    negative_prompt: Option<String>,
    seed: Option<i64>,
}

impl InputVeo2Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputVeo2AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputVeo2Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputVeo2StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputVeo2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputVeo2Builder::prompt)
    pub fn build(self) -> Result<InputVeo2, BuildError> {
        Ok(InputVeo2 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            duration_ms: self.duration_ms,
            start_image: self.start_image,
            negative_prompt: self.negative_prompt,
            seed: self.seed,
        })
    }
}
