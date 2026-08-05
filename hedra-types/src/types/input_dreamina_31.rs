pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `dreamina-31`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InputDreamina31 {
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
    pub aspect_ratio: InputDreamina31AspectRatio,
    /// Output resolution.
    pub resolution: InputDreamina31Resolution,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputDreamina31 {
    pub fn builder() -> InputDreamina31Builder {
        <InputDreamina31Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputDreamina31Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputDreamina31AspectRatio>,
    resolution: Option<InputDreamina31Resolution>,
    seed: Option<i64>,
}

impl InputDreamina31Builder {
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

    pub fn aspect_ratio(mut self, value: InputDreamina31AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputDreamina31Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputDreamina31`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputDreamina31Builder::prompt)
    /// - [`aspect_ratio`](InputDreamina31Builder::aspect_ratio)
    /// - [`resolution`](InputDreamina31Builder::resolution)
    pub fn build(self) -> Result<InputDreamina31, BuildError> {
        Ok(InputDreamina31 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            seed: self.seed,
        })
    }
}
