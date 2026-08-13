pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `recraft-v3`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InputRecraftV3 {
    /// Generation prompt. At most 1000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputRecraftV3AspectRatio,
    /// Output resolution.
    pub resolution: InputRecraftV3Resolution,
}

impl InputRecraftV3 {
    pub fn builder() -> InputRecraftV3Builder {
        <InputRecraftV3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputRecraftV3Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputRecraftV3AspectRatio>,
    resolution: Option<InputRecraftV3Resolution>,
}

impl InputRecraftV3Builder {
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

    pub fn aspect_ratio(mut self, value: InputRecraftV3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputRecraftV3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputRecraftV3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputRecraftV3Builder::prompt)
    /// - [`aspect_ratio`](InputRecraftV3Builder::aspect_ratio)
    /// - [`resolution`](InputRecraftV3Builder::resolution)
    pub fn build(self) -> Result<InputRecraftV3, BuildError> {
        Ok(InputRecraftV3 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
        })
    }
}
