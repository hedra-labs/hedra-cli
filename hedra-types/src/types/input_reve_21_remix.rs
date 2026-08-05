pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `reve-21-remix`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputReve21Remix {
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
    pub aspect_ratio: InputReve21RemixAspectRatio,
    /// Images to edit or blend.
    #[serde(default)]
    pub images: Vec<InputReve21RemixImagesItem>,
    /// Output image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<InputReve21RemixOutputFormat>,
}

impl InputReve21Remix {
    pub fn builder() -> InputReve21RemixBuilder {
        <InputReve21RemixBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputReve21RemixBuilder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputReve21RemixAspectRatio>,
    images: Option<Vec<InputReve21RemixImagesItem>>,
    output_format: Option<InputReve21RemixOutputFormat>,
}

impl InputReve21RemixBuilder {
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

    pub fn aspect_ratio(mut self, value: InputReve21RemixAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputReve21RemixImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn output_format(mut self, value: InputReve21RemixOutputFormat) -> Self {
        self.output_format = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputReve21Remix`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputReve21RemixBuilder::prompt)
    /// - [`aspect_ratio`](InputReve21RemixBuilder::aspect_ratio)
    /// - [`images`](InputReve21RemixBuilder::images)
    pub fn build(self) -> Result<InputReve21Remix, BuildError> {
        Ok(InputReve21Remix {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            images: self.images.ok_or_else(|| BuildError::missing_field("images"))?,
            output_format: self.output_format,
        })
    }
}
