pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `ideogram-v4-5`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: images, prompt; must omit: aspect_ratio, resolution
/// (2) requires: aspect_ratio, prompt, resolution; must omit: images
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputIdeogramV45 {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Images to edit or blend. 1 to 5 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputIdeogramV45ImagesItem>>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputIdeogramV45AspectRatio>,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputIdeogramV45Resolution>,
    /// Quality level to generate at. `low` — the least rendering effort, for quick exploration. `medium` — middle rendering effort, the everyday choice. `high` — the most rendering effort, for poster-ready text and layout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputIdeogramV45Quality>,
}

impl InputIdeogramV45 {
    pub fn builder() -> InputIdeogramV45Builder {
        <InputIdeogramV45Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputIdeogramV45Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    images: Option<Vec<InputIdeogramV45ImagesItem>>,
    seed: Option<i64>,
    aspect_ratio: Option<InputIdeogramV45AspectRatio>,
    resolution: Option<InputIdeogramV45Resolution>,
    quality: Option<InputIdeogramV45Quality>,
}

impl InputIdeogramV45Builder {
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

    pub fn images(mut self, value: Vec<InputIdeogramV45ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputIdeogramV45AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputIdeogramV45Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn quality(mut self, value: InputIdeogramV45Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputIdeogramV45`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputIdeogramV45Builder::prompt)
    pub fn build(self) -> Result<InputIdeogramV45, BuildError> {
        Ok(InputIdeogramV45 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            images: self.images,
            seed: self.seed,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution,
            quality: self.quality,
        })
    }
}
