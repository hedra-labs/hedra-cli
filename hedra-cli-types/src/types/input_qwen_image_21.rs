pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `qwen-image-2-1`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, images, prompt, resolution
/// (2) requires: aspect_ratio, prompt, resolution; must omit: images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputQwenImage21 {
    /// Generation prompt. At least 1 character.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputQwenImage21AspectRatio,
    /// Output resolution.
    pub resolution: InputQwenImage21Resolution,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputQwenImage21OutputFormat>,
    /// What to avoid in the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Images to edit or blend. 1 to 10 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputQwenImage21ImagesItem>>,
    /// Seed for reproducible output; omit for a random seed. From 0 to 2147483646.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// How closely the model follows the prompt. From 1 to 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub guidance_scale: Option<f64>,
}

impl InputQwenImage21 {
    pub fn builder() -> InputQwenImage21Builder {
        <InputQwenImage21Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputQwenImage21Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputQwenImage21AspectRatio>,
    resolution: Option<InputQwenImage21Resolution>,
    output_format: Option<InputQwenImage21OutputFormat>,
    negative_prompt: Option<String>,
    images: Option<Vec<InputQwenImage21ImagesItem>>,
    seed: Option<i64>,
    guidance_scale: Option<f64>,
}

impl InputQwenImage21Builder {
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

    pub fn aspect_ratio(mut self, value: InputQwenImage21AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputQwenImage21Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputQwenImage21OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn images(mut self, value: Vec<InputQwenImage21ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn guidance_scale(mut self, value: f64) -> Self {
        self.guidance_scale = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputQwenImage21`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputQwenImage21Builder::prompt)
    /// - [`aspect_ratio`](InputQwenImage21Builder::aspect_ratio)
    /// - [`resolution`](InputQwenImage21Builder::resolution)
    pub fn build(self) -> Result<InputQwenImage21, BuildError> {
        Ok(InputQwenImage21 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            output_format: self.output_format,
            negative_prompt: self.negative_prompt,
            images: self.images,
            seed: self.seed,
            guidance_scale: self.guidance_scale,
        })
    }
}
