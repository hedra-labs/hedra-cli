pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux2-flex`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, images, prompt
/// (2) requires: aspect_ratio, prompt; must omit: images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux2Flex {
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
    pub aspect_ratio: InputFlux2FlexAspectRatio,
    /// Images to edit or blend. 1 to 8 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputFlux2FlexImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputFlux2FlexOutputFormat>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// How closely the model follows the prompt. From 1.5 to 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub guidance: Option<f64>,
    /// Denoising steps to run. From 2 to 50.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<i64>,
}

impl InputFlux2Flex {
    pub fn builder() -> InputFlux2FlexBuilder {
        <InputFlux2FlexBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux2FlexBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputFlux2FlexAspectRatio>,
    images: Option<Vec<InputFlux2FlexImagesItem>>,
    output_format: Option<InputFlux2FlexOutputFormat>,
    seed: Option<i64>,
    guidance: Option<f64>,
    steps: Option<i64>,
}

impl InputFlux2FlexBuilder {
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

    pub fn aspect_ratio(mut self, value: InputFlux2FlexAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputFlux2FlexImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputFlux2FlexOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn guidance(mut self, value: f64) -> Self {
        self.guidance = Some(value);
        self
    }

    pub fn steps(mut self, value: i64) -> Self {
        self.steps = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFlux2Flex`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFlux2FlexBuilder::prompt)
    /// - [`aspect_ratio`](InputFlux2FlexBuilder::aspect_ratio)
    pub fn build(self) -> Result<InputFlux2Flex, BuildError> {
        Ok(InputFlux2Flex {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images,
            output_format: self.output_format,
            seed: self.seed,
            guidance: self.guidance,
            steps: self.steps,
        })
    }
}
