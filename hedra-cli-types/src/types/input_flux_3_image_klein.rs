pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux-3-image-klein`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux3ImageKlein {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Images to edit or blend. 1 to 10 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputFlux3ImageKleinImagesItem>>,
    /// Output aspect ratio.
    pub aspect_ratio: InputFlux3ImageKleinAspectRatio,
    /// Output resolution.
    pub resolution: InputFlux3ImageKleinResolution,
    /// Seed for reproducible output; omit for a random seed. From 0 to 4294967295.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputFlux3ImageKlein {
    pub fn builder() -> InputFlux3ImageKleinBuilder {
        <InputFlux3ImageKleinBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux3ImageKleinBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    images: Option<Vec<InputFlux3ImageKleinImagesItem>>,
    aspect_ratio: Option<InputFlux3ImageKleinAspectRatio>,
    resolution: Option<InputFlux3ImageKleinResolution>,
    seed: Option<i64>,
}

impl InputFlux3ImageKleinBuilder {
    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn enhance_prompt(mut self, value: bool) -> Self {
        self.enhance_prompt = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputFlux3ImageKleinImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputFlux3ImageKleinAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputFlux3ImageKleinResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFlux3ImageKlein`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFlux3ImageKleinBuilder::prompt)
    /// - [`aspect_ratio`](InputFlux3ImageKleinBuilder::aspect_ratio)
    /// - [`resolution`](InputFlux3ImageKleinBuilder::resolution)
    pub fn build(self) -> Result<InputFlux3ImageKlein, BuildError> {
        Ok(InputFlux3ImageKlein {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            images: self.images,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            seed: self.seed,
        })
    }
}
