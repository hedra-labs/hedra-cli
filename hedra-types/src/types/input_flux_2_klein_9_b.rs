pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux2-klein-9b`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux2Klein9B {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputFlux2Klein9BAspectRatio,
    /// Images to edit or blend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputFlux2Klein9BImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputFlux2Klein9BOutputFormat>,
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

impl InputFlux2Klein9B {
    pub fn builder() -> InputFlux2Klein9BBuilder {
        <InputFlux2Klein9BBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux2Klein9BBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputFlux2Klein9BAspectRatio>,
    images: Option<Vec<InputFlux2Klein9BImagesItem>>,
    output_format: Option<InputFlux2Klein9BOutputFormat>,
    negative_prompt: Option<String>,
    seed: Option<i64>,
    guidance_scale: Option<f64>,
    num_inference_steps: Option<i64>,
}

impl InputFlux2Klein9BBuilder {
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

    pub fn aspect_ratio(mut self, value: InputFlux2Klein9BAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputFlux2Klein9BImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputFlux2Klein9BOutputFormat) -> Self {
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

    /// Consumes the builder and constructs a [`InputFlux2Klein9B`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFlux2Klein9BBuilder::prompt)
    /// - [`aspect_ratio`](InputFlux2Klein9BBuilder::aspect_ratio)
    pub fn build(self) -> Result<InputFlux2Klein9B, BuildError> {
        Ok(InputFlux2Klein9B {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images,
            output_format: self.output_format,
            negative_prompt: self.negative_prompt,
            seed: self.seed,
            guidance_scale: self.guidance_scale,
            num_inference_steps: self.num_inference_steps,
        })
    }
}
