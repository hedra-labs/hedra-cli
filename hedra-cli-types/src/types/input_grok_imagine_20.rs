pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `grok-imagine-20`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGrokImagine20 {
    /// Generation prompt. At most 8000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Images to edit or blend; omit to render from the prompt alone. 1 to 3 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGrokImagine20ImagesItem>>,
    /// Output aspect ratio.
    pub aspect_ratio: InputGrokImagine20AspectRatio,
    /// Output resolution.
    pub resolution: InputGrokImagine20Resolution,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputGrokImagine20OutputFormat>,
}

impl InputGrokImagine20 {
    pub fn builder() -> InputGrokImagine20Builder {
        <InputGrokImagine20Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGrokImagine20Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    images: Option<Vec<InputGrokImagine20ImagesItem>>,
    aspect_ratio: Option<InputGrokImagine20AspectRatio>,
    resolution: Option<InputGrokImagine20Resolution>,
    output_format: Option<InputGrokImagine20OutputFormat>,
}

impl InputGrokImagine20Builder {
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

    pub fn images(mut self, value: Vec<InputGrokImagine20ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputGrokImagine20AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGrokImagine20Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputGrokImagine20OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGrokImagine20`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGrokImagine20Builder::prompt)
    /// - [`aspect_ratio`](InputGrokImagine20Builder::aspect_ratio)
    /// - [`resolution`](InputGrokImagine20Builder::resolution)
    pub fn build(self) -> Result<InputGrokImagine20, BuildError> {
        Ok(InputGrokImagine20 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            images: self.images,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            output_format: self.output_format,
        })
    }
}
