pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `reve-21`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InputReve21 {
    /// Generation prompt. From 1 to 4000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputReve21AspectRatio,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputReve21OutputFormat>,
}

impl InputReve21 {
    pub fn builder() -> InputReve21Builder {
        <InputReve21Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputReve21Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputReve21AspectRatio>,
    output_format: Option<InputReve21OutputFormat>,
}

impl InputReve21Builder {
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

    pub fn aspect_ratio(mut self, value: InputReve21AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputReve21OutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputReve21`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputReve21Builder::prompt)
    /// - [`aspect_ratio`](InputReve21Builder::aspect_ratio)
    pub fn build(self) -> Result<InputReve21, BuildError> {
        Ok(InputReve21 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            output_format: self.output_format,
        })
    }
}
