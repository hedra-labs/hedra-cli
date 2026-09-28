pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `minimax-h3-ultra`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: duration_ms, prompt, start_image; must omit: aspect_ratio, end_image
/// (2) requires: duration_ms, end_image, prompt, start_image; must omit: aspect_ratio
/// (3) requires: aspect_ratio, duration_ms, prompt; must omit: end_image, start_image
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputMinimaxH3Ultra {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. From 1 to 7000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputMinimaxH3UltraResolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Start frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputMinimaxH3UltraStartImage>,
    /// End frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputMinimaxH3UltraEndImage>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputMinimaxH3UltraAspectRatio>,
}

impl InputMinimaxH3Ultra {
    pub fn builder() -> InputMinimaxH3UltraBuilder {
        <InputMinimaxH3UltraBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMinimaxH3UltraBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputMinimaxH3UltraResolution>,
    duration_ms: Option<i64>,
    enhance_prompt: Option<bool>,
    start_image: Option<InputMinimaxH3UltraStartImage>,
    end_image: Option<InputMinimaxH3UltraEndImage>,
    aspect_ratio: Option<InputMinimaxH3UltraAspectRatio>,
}

impl InputMinimaxH3UltraBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputMinimaxH3UltraResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn enhance_prompt(mut self, value: bool) -> Self {
        self.enhance_prompt = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputMinimaxH3UltraStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputMinimaxH3UltraEndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputMinimaxH3UltraAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMinimaxH3Ultra`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputMinimaxH3UltraBuilder::prompt)
    /// - [`duration_ms`](InputMinimaxH3UltraBuilder::duration_ms)
    pub fn build(self) -> Result<InputMinimaxH3Ultra, BuildError> {
        Ok(InputMinimaxH3Ultra {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            enhance_prompt: self.enhance_prompt,
            start_image: self.start_image,
            end_image: self.end_image,
            aspect_ratio: self.aspect_ratio,
        })
    }
}
