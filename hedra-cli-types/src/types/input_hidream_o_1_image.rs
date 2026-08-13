pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `hidream-o1-image`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, images, prompt; must omit: resolution
/// (2) requires: aspect_ratio, prompt, resolution; must omit: images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputHidreamO1Image {
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
    pub aspect_ratio: InputHidreamO1ImageAspectRatio,
    /// Images to edit or blend. 1 to 10 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputHidreamO1ImageImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputHidreamO1ImageOutputFormat>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// How closely the model follows the prompt. From 0 to 20.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub guidance_scale: Option<f64>,
    /// Denoising steps to run. From 1 to 50.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_inference_steps: Option<i64>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputHidreamO1ImageResolution>,
    /// Quality level to generate at.
    pub quality: InputHidreamO1ImageQuality,
}

impl InputHidreamO1Image {
    pub fn builder() -> InputHidreamO1ImageBuilder {
        <InputHidreamO1ImageBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputHidreamO1ImageBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputHidreamO1ImageAspectRatio>,
    images: Option<Vec<InputHidreamO1ImageImagesItem>>,
    output_format: Option<InputHidreamO1ImageOutputFormat>,
    seed: Option<i64>,
    guidance_scale: Option<f64>,
    num_inference_steps: Option<i64>,
    resolution: Option<InputHidreamO1ImageResolution>,
    quality: Option<InputHidreamO1ImageQuality>,
}

impl InputHidreamO1ImageBuilder {
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

    pub fn aspect_ratio(mut self, value: InputHidreamO1ImageAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputHidreamO1ImageImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputHidreamO1ImageOutputFormat) -> Self {
        self.output_format = Some(value);
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

    pub fn resolution(mut self, value: InputHidreamO1ImageResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn quality(mut self, value: InputHidreamO1ImageQuality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputHidreamO1Image`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputHidreamO1ImageBuilder::prompt)
    /// - [`aspect_ratio`](InputHidreamO1ImageBuilder::aspect_ratio)
    /// - [`quality`](InputHidreamO1ImageBuilder::quality)
    pub fn build(self) -> Result<InputHidreamO1Image, BuildError> {
        Ok(InputHidreamO1Image {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images,
            output_format: self.output_format,
            seed: self.seed,
            guidance_scale: self.guidance_scale,
            num_inference_steps: self.num_inference_steps,
            resolution: self.resolution,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
