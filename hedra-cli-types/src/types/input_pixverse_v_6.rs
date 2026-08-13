pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `pixverse-v6`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: duration_ms, prompt, resolution, start_image; must omit: aspect_ratio
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputPixverseV6 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2048 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    pub resolution: InputPixverseV6Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// What to avoid in the generated video. At most 2048 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Start frame (image-to-video). At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputPixverseV6StartImage>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputPixverseV6AspectRatio>,
}

impl InputPixverseV6 {
    pub fn builder() -> InputPixverseV6Builder {
        <InputPixverseV6Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputPixverseV6Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputPixverseV6Resolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    negative_prompt: Option<String>,
    seed: Option<i64>,
    start_image: Option<InputPixverseV6StartImage>,
    aspect_ratio: Option<InputPixverseV6AspectRatio>,
}

impl InputPixverseV6Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputPixverseV6Resolution) -> Self {
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

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputPixverseV6StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputPixverseV6AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputPixverseV6`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputPixverseV6Builder::prompt)
    /// - [`resolution`](InputPixverseV6Builder::resolution)
    /// - [`duration_ms`](InputPixverseV6Builder::duration_ms)
    pub fn build(self) -> Result<InputPixverseV6, BuildError> {
        Ok(InputPixverseV6 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            generate_audio: self.generate_audio,
            negative_prompt: self.negative_prompt,
            seed: self.seed,
            start_image: self.start_image,
            aspect_ratio: self.aspect_ratio,
        })
    }
}
