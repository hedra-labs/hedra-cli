pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `ideogram-v4`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InputIdeogramV4 {
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
    pub aspect_ratio: InputIdeogramV4AspectRatio,
    /// Output resolution.
    pub resolution: InputIdeogramV4Resolution,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputIdeogramV4OutputFormat>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Quality level to generate at.
    pub quality: InputIdeogramV4Quality,
}

impl InputIdeogramV4 {
    pub fn builder() -> InputIdeogramV4Builder {
        <InputIdeogramV4Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputIdeogramV4Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputIdeogramV4AspectRatio>,
    resolution: Option<InputIdeogramV4Resolution>,
    output_format: Option<InputIdeogramV4OutputFormat>,
    seed: Option<i64>,
    quality: Option<InputIdeogramV4Quality>,
}

impl InputIdeogramV4Builder {
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

    pub fn aspect_ratio(mut self, value: InputIdeogramV4AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputIdeogramV4Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputIdeogramV4OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn quality(mut self, value: InputIdeogramV4Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputIdeogramV4`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputIdeogramV4Builder::prompt)
    /// - [`aspect_ratio`](InputIdeogramV4Builder::aspect_ratio)
    /// - [`resolution`](InputIdeogramV4Builder::resolution)
    /// - [`quality`](InputIdeogramV4Builder::quality)
    pub fn build(self) -> Result<InputIdeogramV4, BuildError> {
        Ok(InputIdeogramV4 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            output_format: self.output_format,
            seed: self.seed,
            quality: self.quality.ok_or_else(|| BuildError::missing_field("quality"))?,
        })
    }
}
