pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `seedream-45`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSeedream45 {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputSeedream45AspectRatio,
    /// Output resolution.
    pub resolution: InputSeedream45Resolution,
    /// Images to edit or blend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputSeedream45ImagesItem>>,
}

impl InputSeedream45 {
    pub fn builder() -> InputSeedream45Builder {
        <InputSeedream45Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputSeedream45Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputSeedream45AspectRatio>,
    resolution: Option<InputSeedream45Resolution>,
    images: Option<Vec<InputSeedream45ImagesItem>>,
}

impl InputSeedream45Builder {
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

    pub fn aspect_ratio(mut self, value: InputSeedream45AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputSeedream45Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputSeedream45ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputSeedream45`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputSeedream45Builder::prompt)
    /// - [`aspect_ratio`](InputSeedream45Builder::aspect_ratio)
    /// - [`resolution`](InputSeedream45Builder::resolution)
    pub fn build(self) -> Result<InputSeedream45, BuildError> {
        Ok(InputSeedream45 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            images: self.images,
        })
    }
}
