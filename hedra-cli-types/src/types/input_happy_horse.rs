pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `happy-horse`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, resolution, start_image
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution; accepts aspect_ratio: 16:9 | 4:3 | 1:1 | 3:4 | 9:16
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputHappyHorse {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputHappyHorseAspectRatio,
    /// Output resolution.
    pub resolution: InputHappyHorseResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame (image-to-video). At least 300px on each side and at most 10 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputHappyHorseStartImage>,
    /// Reference images. 1 to 9 images, each at least 400px on each side and at most 10 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputHappyHorseImagesItem>>,
    /// Seed for reproducible output; omit for a random seed. From 0 to 2147483647.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputHappyHorse {
    pub fn builder() -> InputHappyHorseBuilder {
        <InputHappyHorseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputHappyHorseBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputHappyHorseAspectRatio>,
    resolution: Option<InputHappyHorseResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputHappyHorseStartImage>,
    images: Option<Vec<InputHappyHorseImagesItem>>,
    seed: Option<i64>,
}

impl InputHappyHorseBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputHappyHorseAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputHappyHorseResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputHappyHorseStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputHappyHorseImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputHappyHorse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputHappyHorseBuilder::prompt)
    /// - [`aspect_ratio`](InputHappyHorseBuilder::aspect_ratio)
    /// - [`resolution`](InputHappyHorseBuilder::resolution)
    /// - [`duration_ms`](InputHappyHorseBuilder::duration_ms)
    pub fn build(self) -> Result<InputHappyHorse, BuildError> {
        Ok(InputHappyHorse {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            images: self.images,
            seed: self.seed,
        })
    }
}
