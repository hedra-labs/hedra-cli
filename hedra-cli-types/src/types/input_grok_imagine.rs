pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `grok-imagine`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: images, prompt; must omit: aspect_ratio, resolution; accepts quality: standard
/// (2) requires: aspect_ratio, images, prompt, resolution; accepts quality: quality
/// (3) requires: aspect_ratio, prompt, resolution; must omit: images; accepts aspect_ratio: 2:1 | 20:9 | 19.5:9 | 16:9 | 4:3 | 3:2 | 1:1 | 2:3 | 3:4 | 9:16 | 9:19.5 | 9:20 | 1:2; quality: quality
/// (4) requires: aspect_ratio, prompt; must omit: images, resolution; accepts aspect_ratio: 2:1 | 20:9 | 19.5:9 | 16:9 | 4:3 | 3:2 | 1:1 | 2:3 | 3:4 | 9:16 | 9:19.5 | 9:20 | 1:2; quality: standard
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputGrokImagine {
    /// Generation prompt. At most 8000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Images to edit or blend. 1 to 3 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGrokImagineImagesItem>>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputGrokImagineOutputFormat>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputGrokImagineAspectRatio>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputGrokImagineResolution>,
    /// Quality level to generate at. `standard` — the base tier, at a flat rate whatever the output size. `quality` — xAI's higher-fidelity tier, and the only one offering 2k.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputGrokImagineQuality>,
}

impl InputGrokImagine {
    pub fn builder() -> InputGrokImagineBuilder {
        <InputGrokImagineBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGrokImagineBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    images: Option<Vec<InputGrokImagineImagesItem>>,
    output_format: Option<InputGrokImagineOutputFormat>,
    aspect_ratio: Option<InputGrokImagineAspectRatio>,
    resolution: Option<InputGrokImagineResolution>,
    quality: Option<InputGrokImagineQuality>,
}

impl InputGrokImagineBuilder {
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

    pub fn images(mut self, value: Vec<InputGrokImagineImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputGrokImagineOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputGrokImagineAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGrokImagineResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn quality(mut self, value: InputGrokImagineQuality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGrokImagine`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGrokImagineBuilder::prompt)
    pub fn build(self) -> Result<InputGrokImagine, BuildError> {
        Ok(InputGrokImagine {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            images: self.images,
            output_format: self.output_format,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            quality: self.quality,
        })
    }
}
