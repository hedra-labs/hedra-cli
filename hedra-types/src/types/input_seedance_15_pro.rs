pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `seedance-15-pro`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSeedance15Pro {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputSeedance15ProAspectRatio,
    /// Output resolution.
    pub resolution: InputSeedance15ProResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputSeedance15ProStartImage>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputSeedance15ProEndImage>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputSeedance15Pro {
    pub fn builder() -> InputSeedance15ProBuilder {
        <InputSeedance15ProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputSeedance15ProBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputSeedance15ProAspectRatio>,
    resolution: Option<InputSeedance15ProResolution>,
    duration_ms: Option<i64>,
    generate_audio: Option<bool>,
    start_image: Option<InputSeedance15ProStartImage>,
    end_image: Option<InputSeedance15ProEndImage>,
    seed: Option<i64>,
}

impl InputSeedance15ProBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputSeedance15ProAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputSeedance15ProResolution) -> Self {
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

    pub fn start_image(mut self, value: InputSeedance15ProStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputSeedance15ProEndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputSeedance15Pro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputSeedance15ProBuilder::prompt)
    /// - [`aspect_ratio`](InputSeedance15ProBuilder::aspect_ratio)
    /// - [`resolution`](InputSeedance15ProBuilder::resolution)
    /// - [`duration_ms`](InputSeedance15ProBuilder::duration_ms)
    pub fn build(self) -> Result<InputSeedance15Pro, BuildError> {
        Ok(InputSeedance15Pro {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            generate_audio: self.generate_audio,
            start_image: self.start_image,
            end_image: self.end_image,
            seed: self.seed,
        })
    }
}
