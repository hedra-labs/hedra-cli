pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux-kontext-pro`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, prompt
/// (2) requires: images, prompt
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputFluxKontextPro {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputFluxKontextProAspectRatio>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputFluxKontextProResolution>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputFluxKontextProOutputFormat>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Images to edit or blend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputFluxKontextProImagesItem>>,
}

impl InputFluxKontextPro {
    pub fn builder() -> InputFluxKontextProBuilder {
        <InputFluxKontextProBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFluxKontextProBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputFluxKontextProAspectRatio>,
    resolution: Option<InputFluxKontextProResolution>,
    output_format: Option<InputFluxKontextProOutputFormat>,
    seed: Option<i64>,
    images: Option<Vec<InputFluxKontextProImagesItem>>,
}

impl InputFluxKontextProBuilder {
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

    pub fn aspect_ratio(mut self, value: InputFluxKontextProAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputFluxKontextProResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputFluxKontextProOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputFluxKontextProImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFluxKontextPro`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFluxKontextProBuilder::prompt)
    pub fn build(self) -> Result<InputFluxKontextPro, BuildError> {
        Ok(InputFluxKontextPro {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            output_format: self.output_format,
            seed: self.seed,
            images: self.images,
        })
    }
}
