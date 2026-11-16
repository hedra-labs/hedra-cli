pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux-3-image`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux3Image {
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
    pub images: Option<Vec<InputFlux3ImageImagesItem>>,
    /// Output aspect ratio.
    pub aspect_ratio: InputFlux3ImageAspectRatio,
    /// Output resolution.
    pub resolution: InputFlux3ImageResolution,
}

impl InputFlux3Image {
    pub fn builder() -> InputFlux3ImageBuilder {
        <InputFlux3ImageBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux3ImageBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    images: Option<Vec<InputFlux3ImageImagesItem>>,
    aspect_ratio: Option<InputFlux3ImageAspectRatio>,
    resolution: Option<InputFlux3ImageResolution>,
}

impl InputFlux3ImageBuilder {
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

    pub fn images(mut self, value: Vec<InputFlux3ImageImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputFlux3ImageAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputFlux3ImageResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFlux3Image`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFlux3ImageBuilder::prompt)
    /// - [`aspect_ratio`](InputFlux3ImageBuilder::aspect_ratio)
    /// - [`resolution`](InputFlux3ImageBuilder::resolution)
    pub fn build(self) -> Result<InputFlux3Image, BuildError> {
        Ok(InputFlux3Image {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            images: self.images,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
        })
    }
}
