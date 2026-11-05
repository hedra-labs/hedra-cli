pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux2`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, images, prompt; accepts quality: flex
/// (2) requires: aspect_ratio, prompt; must omit: images; accepts quality: flex
/// (3) requires: aspect_ratio, images, prompt; must omit: guidance, steps; accepts quality: pro | max
/// (4) requires: aspect_ratio, prompt; must omit: guidance, images, steps; accepts quality: pro | max
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux2 {
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
    pub aspect_ratio: InputFlux2AspectRatio,
    /// Images to edit or blend. 1 to 8 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputFlux2ImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputFlux2OutputFormat>,
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
    /// Quality level to generate at. `pro` — the everyday tier, for style transfer and sequential edits. `max` — the top tier, for realism and precision in final deliverables, at the highest price before reference images. `flex` — the tunable tier, adding `guidance` and `steps` to trade fidelity against speed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputFlux2Quality>,
}

impl InputFlux2 {
    pub fn builder() -> InputFlux2Builder {
        <InputFlux2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux2Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputFlux2AspectRatio>,
    images: Option<Vec<InputFlux2ImagesItem>>,
    output_format: Option<InputFlux2OutputFormat>,
    seed: Option<i64>,
    guidance: Option<f64>,
    steps: Option<i64>,
    quality: Option<InputFlux2Quality>,
}

impl InputFlux2Builder {
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

    pub fn aspect_ratio(mut self, value: InputFlux2AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputFlux2ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputFlux2OutputFormat) -> Self {
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

    pub fn quality(mut self, value: InputFlux2Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFlux2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputFlux2Builder::prompt)
    /// - [`aspect_ratio`](InputFlux2Builder::aspect_ratio)
    pub fn build(self) -> Result<InputFlux2, BuildError> {
        Ok(InputFlux2 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images,
            output_format: self.output_format,
            seed: self.seed,
            guidance: self.guidance,
            steps: self.steps,
            quality: self.quality,
        })
    }
}
