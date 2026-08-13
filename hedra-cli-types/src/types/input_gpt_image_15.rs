pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `gpt-image-15`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, images, prompt
/// (2) requires: aspect_ratio, prompt; must omit: images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGptImage15 {
    /// Generation prompt. At most 32000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputGptImage15AspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputGptImage15Resolution>,
    /// Images to edit or blend. 1 to 16 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGptImage15ImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputGptImage15OutputFormat>,
}

impl InputGptImage15 {
    pub fn builder() -> InputGptImage15Builder {
        <InputGptImage15Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGptImage15Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputGptImage15AspectRatio>,
    resolution: Option<InputGptImage15Resolution>,
    images: Option<Vec<InputGptImage15ImagesItem>>,
    output_format: Option<InputGptImage15OutputFormat>,
}

impl InputGptImage15Builder {
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

    pub fn aspect_ratio(mut self, value: InputGptImage15AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGptImage15Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputGptImage15ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputGptImage15OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGptImage15`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGptImage15Builder::prompt)
    /// - [`aspect_ratio`](InputGptImage15Builder::aspect_ratio)
    pub fn build(self) -> Result<InputGptImage15, BuildError> {
        Ok(InputGptImage15 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            images: self.images,
            output_format: self.output_format,
        })
    }
}
