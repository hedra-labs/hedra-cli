pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux2-pro`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux2Pro {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputFlux2ProAspectRatio,
    /// Images to edit or blend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputFlux2ProImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputFlux2ProOutputFormat>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputFlux2Pro {
    pub fn builder() -> InputFlux2ProBuilder {
        <InputFlux2ProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux2ProBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputFlux2ProAspectRatio>,
    images: Option<Vec<InputFlux2ProImagesItem>>,
    output_format: Option<InputFlux2ProOutputFormat>,
    seed: Option<i64>,
}

impl InputFlux2ProBuilder {
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

    pub fn aspect_ratio(mut self, value: InputFlux2ProAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputFlux2ProImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputFlux2ProOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFlux2Pro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFlux2ProBuilder::prompt)
    /// - [`aspect_ratio`](InputFlux2ProBuilder::aspect_ratio)
    pub fn build(self) -> Result<InputFlux2Pro, BuildError> {
        Ok(InputFlux2Pro {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images,
            output_format: self.output_format,
            seed: self.seed,
        })
    }
}
