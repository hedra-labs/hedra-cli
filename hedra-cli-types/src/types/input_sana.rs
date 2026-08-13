pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `sana`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputSana {
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
    pub aspect_ratio: InputSanaAspectRatio,
    /// Output resolution.
    pub resolution: InputSanaResolution,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputSanaOutputFormat>,
    /// What to avoid in the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// How closely the model follows the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub guidance_scale: Option<f64>,
    /// Denoising steps to run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_inference_steps: Option<i64>,
}

impl InputSana {
    pub fn builder() -> InputSanaBuilder {
        <InputSanaBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputSanaBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputSanaAspectRatio>,
    resolution: Option<InputSanaResolution>,
    output_format: Option<InputSanaOutputFormat>,
    negative_prompt: Option<String>,
    seed: Option<i64>,
    guidance_scale: Option<f64>,
    num_inference_steps: Option<i64>,
}

impl InputSanaBuilder {
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

    pub fn aspect_ratio(mut self, value: InputSanaAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputSanaResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputSanaOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
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

    pub fn num_inference_steps(mut self, value: i64) -> Self {
        self.num_inference_steps = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputSana`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputSanaBuilder::prompt)
    /// - [`aspect_ratio`](InputSanaBuilder::aspect_ratio)
    /// - [`resolution`](InputSanaBuilder::resolution)
    pub fn build(self) -> Result<InputSana, BuildError> {
        Ok(InputSana {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            output_format: self.output_format,
            negative_prompt: self.negative_prompt,
            seed: self.seed,
            guidance_scale: self.guidance_scale,
            num_inference_steps: self.num_inference_steps,
        })
    }
}
