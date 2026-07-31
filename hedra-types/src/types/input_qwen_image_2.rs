pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `qwen-image-2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputQwenImage2 {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputQwenImage2AspectRatio,
    /// Output resolution.
    pub resolution: InputQwenImage2Resolution,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputQwenImage2OutputFormat>,
    /// What to avoid in the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Images to edit or blend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputQwenImage2ImagesItem>>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Quality level to generate at.
    pub quality: InputQwenImage2Quality,
}

impl InputQwenImage2 {
    pub fn builder() -> InputQwenImage2Builder {
        <InputQwenImage2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputQwenImage2Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputQwenImage2AspectRatio>,
    resolution: Option<InputQwenImage2Resolution>,
    output_format: Option<InputQwenImage2OutputFormat>,
    negative_prompt: Option<String>,
    images: Option<Vec<InputQwenImage2ImagesItem>>,
    seed: Option<i64>,
    quality: Option<InputQwenImage2Quality>,
}

impl InputQwenImage2Builder {
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

    pub fn aspect_ratio(mut self, value: InputQwenImage2AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputQwenImage2Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputQwenImage2OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn images(mut self, value: Vec<InputQwenImage2ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn quality(mut self, value: InputQwenImage2Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputQwenImage2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputQwenImage2Builder::prompt)
    /// - [`aspect_ratio`](InputQwenImage2Builder::aspect_ratio)
    /// - [`resolution`](InputQwenImage2Builder::resolution)
    /// - [`quality`](InputQwenImage2Builder::quality)
    pub fn build(self) -> Result<InputQwenImage2, BuildError> {
        Ok(InputQwenImage2 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            output_format: self.output_format,
            negative_prompt: self.negative_prompt,
            images: self.images,
            seed: self.seed,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
