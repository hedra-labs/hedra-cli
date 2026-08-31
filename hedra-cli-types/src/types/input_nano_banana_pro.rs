pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `nano-banana-pro`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputNanoBananaPro {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio. 'adaptive' lets the model size the output itself — matching the source image when you pass one.
    pub aspect_ratio: InputNanoBananaProAspectRatio,
    /// Output resolution.
    pub resolution: InputNanoBananaProResolution,
    /// Images to edit or blend, at most 6 high-fidelity objects, 5 characters and 3 style references. 1 to 14 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputNanoBananaProImagesItem>>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Ground the generation in live Google Search results, so a prompt about current events or real-world specifics draws on what the web says now. Grounded generations cost more than ungrounded ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<bool>,
}

impl InputNanoBananaPro {
    pub fn builder() -> InputNanoBananaProBuilder {
        <InputNanoBananaProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputNanoBananaProBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputNanoBananaProAspectRatio>,
    resolution: Option<InputNanoBananaProResolution>,
    images: Option<Vec<InputNanoBananaProImagesItem>>,
    seed: Option<i64>,
    google_search: Option<bool>,
}

impl InputNanoBananaProBuilder {
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

    pub fn aspect_ratio(mut self, value: InputNanoBananaProAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputNanoBananaProResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputNanoBananaProImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn google_search(mut self, value: bool) -> Self {
        self.google_search = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputNanoBananaPro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputNanoBananaProBuilder::prompt)
    /// - [`aspect_ratio`](InputNanoBananaProBuilder::aspect_ratio)
    /// - [`resolution`](InputNanoBananaProBuilder::resolution)
    pub fn build(self) -> Result<InputNanoBananaPro, BuildError> {
        Ok(InputNanoBananaPro {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            images: self.images,
            seed: self.seed,
            google_search: self.google_search,
        })
    }
}
