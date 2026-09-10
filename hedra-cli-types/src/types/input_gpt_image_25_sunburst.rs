pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `gpt-image-2.5-sunburst`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, images, prompt, resolution
/// (2) requires: aspect_ratio, prompt, resolution; must omit: images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputGptImage25Sunburst {
    /// Generation prompt. At most 32000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputGptImage25SunburstAspectRatio,
    /// Output resolution.
    pub resolution: InputGptImage25SunburstResolution,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputGptImage25SunburstOutputFormat>,
    /// Images to edit or blend. 1 to 16 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputGptImage25SunburstImagesItem>>,
    /// Quality level to generate at. `auto` — automatically selects rendering quality for the prompt. `low` — low rendering quality for the Sunburst model. `medium` — medium rendering quality for the Sunburst model. `high` — high rendering quality for the Sunburst model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputGptImage25SunburstQuality>,
}

impl InputGptImage25Sunburst {
    pub fn builder() -> InputGptImage25SunburstBuilder {
        <InputGptImage25SunburstBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputGptImage25SunburstBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputGptImage25SunburstAspectRatio>,
    resolution: Option<InputGptImage25SunburstResolution>,
    output_format: Option<InputGptImage25SunburstOutputFormat>,
    images: Option<Vec<InputGptImage25SunburstImagesItem>>,
    quality: Option<InputGptImage25SunburstQuality>,
}

impl InputGptImage25SunburstBuilder {
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

    pub fn aspect_ratio(mut self, value: InputGptImage25SunburstAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputGptImage25SunburstResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputGptImage25SunburstOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputGptImage25SunburstImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn quality(mut self, value: InputGptImage25SunburstQuality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputGptImage25Sunburst`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputGptImage25SunburstBuilder::prompt)
    /// - [`aspect_ratio`](InputGptImage25SunburstBuilder::aspect_ratio)
    /// - [`resolution`](InputGptImage25SunburstBuilder::resolution)
    pub fn build(self) -> Result<InputGptImage25Sunburst, BuildError> {
        Ok(InputGptImage25Sunburst {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            output_format: self.output_format,
            images: self.images,
            quality: self.quality,
        })
    }
}
