pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `gpt-image-2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGptImage2 {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputGptImage2AspectRatio,
    /// Output resolution.
    pub resolution: InputGptImage2Resolution,
    /// Images to edit or blend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGptImage2ImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputGptImage2OutputFormat>,
    /// Quality level to generate at.
    pub quality: InputGptImage2Quality,
}

impl InputGptImage2 {
    pub fn builder() -> InputGptImage2Builder {
        <InputGptImage2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGptImage2Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputGptImage2AspectRatio>,
    resolution: Option<InputGptImage2Resolution>,
    images: Option<Vec<InputGptImage2ImagesItem>>,
    output_format: Option<InputGptImage2OutputFormat>,
    quality: Option<InputGptImage2Quality>,
}

impl InputGptImage2Builder {
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

    pub fn aspect_ratio(mut self, value: InputGptImage2AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGptImage2Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputGptImage2ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputGptImage2OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn quality(mut self, value: InputGptImage2Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGptImage2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGptImage2Builder::prompt)
    /// - [`aspect_ratio`](InputGptImage2Builder::aspect_ratio)
    /// - [`resolution`](InputGptImage2Builder::resolution)
    /// - [`quality`](InputGptImage2Builder::quality)
    pub fn build(self) -> Result<InputGptImage2, BuildError> {
        Ok(InputGptImage2 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            images: self.images,
            output_format: self.output_format,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
