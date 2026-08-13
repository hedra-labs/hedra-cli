pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `nano-banana-2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputNanoBanana2 {
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
    pub aspect_ratio: InputNanoBanana2AspectRatio,
    /// Output resolution.
    pub resolution: InputNanoBanana2Resolution,
    /// Images to edit or blend, at most 10 high-fidelity objects and 4 characters. 1 to 14 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputNanoBanana2ImagesItem>>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputNanoBanana2 {
    pub fn builder() -> InputNanoBanana2Builder {
        <InputNanoBanana2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputNanoBanana2Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputNanoBanana2AspectRatio>,
    resolution: Option<InputNanoBanana2Resolution>,
    images: Option<Vec<InputNanoBanana2ImagesItem>>,
    seed: Option<i64>,
}

impl InputNanoBanana2Builder {
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

    pub fn aspect_ratio(mut self, value: InputNanoBanana2AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputNanoBanana2Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputNanoBanana2ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputNanoBanana2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputNanoBanana2Builder::prompt)
    /// - [`aspect_ratio`](InputNanoBanana2Builder::aspect_ratio)
    /// - [`resolution`](InputNanoBanana2Builder::resolution)
    pub fn build(self) -> Result<InputNanoBanana2, BuildError> {
        Ok(InputNanoBanana2 {
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
