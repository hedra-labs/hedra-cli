pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `reve-21-edit`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputReve21Edit {
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
    pub aspect_ratio: InputReve21EditAspectRatio,
    /// The single source image to edit.
    #[serde(default)]
    pub images: Vec<InputReve21EditImagesItem>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputReve21EditOutputFormat>,
}

impl InputReve21Edit {
    pub fn builder() -> InputReve21EditBuilder {
        <InputReve21EditBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputReve21EditBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputReve21EditAspectRatio>,
    images: Option<Vec<InputReve21EditImagesItem>>,
    output_format: Option<InputReve21EditOutputFormat>,
}

impl InputReve21EditBuilder {
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

    pub fn aspect_ratio(mut self, value: InputReve21EditAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputReve21EditImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputReve21EditOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputReve21Edit`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputReve21EditBuilder::prompt)
    /// - [`aspect_ratio`](InputReve21EditBuilder::aspect_ratio)
    /// - [`images`](InputReve21EditBuilder::images)
    pub fn build(self) -> Result<InputReve21Edit, BuildError> {
        Ok(InputReve21Edit {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images.ok_or_else(|| BuildError::missing_field("images"))?,
            output_format: self.output_format,
        })
    }
}
