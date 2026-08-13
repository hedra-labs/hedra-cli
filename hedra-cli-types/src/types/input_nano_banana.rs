pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `nano-banana`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputNanoBanana {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputNanoBananaAspectRatio,
    /// Output resolution.
    pub resolution: InputNanoBananaResolution,
    /// Images to edit or blend. 1 to 3 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputNanoBananaImagesItem>>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputNanoBanana {
    pub fn builder() -> InputNanoBananaBuilder {
        <InputNanoBananaBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputNanoBananaBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputNanoBananaAspectRatio>,
    resolution: Option<InputNanoBananaResolution>,
    images: Option<Vec<InputNanoBananaImagesItem>>,
    seed: Option<i64>,
}

impl InputNanoBananaBuilder {
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

    pub fn aspect_ratio(mut self, value: InputNanoBananaAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputNanoBananaResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputNanoBananaImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputNanoBanana`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputNanoBananaBuilder::prompt)
    /// - [`aspect_ratio`](InputNanoBananaBuilder::aspect_ratio)
    /// - [`resolution`](InputNanoBananaBuilder::resolution)
    pub fn build(self) -> Result<InputNanoBanana, BuildError> {
        Ok(InputNanoBanana {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            images: self.images,
            seed: self.seed,
        })
    }
}
