pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `mai-image-2-5`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, images, prompt
/// (2) requires: aspect_ratio, prompt; must omit: images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMaiImage25 {
    /// Generation prompt. From 3 to 5000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputMaiImage25AspectRatio,
    /// The single source image to edit. Exactly 1 image, at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputMaiImage25ImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputMaiImage25OutputFormat>,
    /// Quality level to generate at. `standard` — photorealistic generation and editing at the base rate. `pro` — the higher-fidelity tier, for final deliverables that need maximum detail and text rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputMaiImage25Quality>,
}

impl InputMaiImage25 {
    pub fn builder() -> InputMaiImage25Builder {
        <InputMaiImage25Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMaiImage25Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputMaiImage25AspectRatio>,
    images: Option<Vec<InputMaiImage25ImagesItem>>,
    output_format: Option<InputMaiImage25OutputFormat>,
    quality: Option<InputMaiImage25Quality>,
}

impl InputMaiImage25Builder {
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

    pub fn aspect_ratio(mut self, value: InputMaiImage25AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputMaiImage25ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputMaiImage25OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn quality(mut self, value: InputMaiImage25Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMaiImage25`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputMaiImage25Builder::prompt)
    /// - [`aspect_ratio`](InputMaiImage25Builder::aspect_ratio)
    pub fn build(self) -> Result<InputMaiImage25, BuildError> {
        Ok(InputMaiImage25 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images,
            output_format: self.output_format,
            quality: self.quality,
        })
    }
}
