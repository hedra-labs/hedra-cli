pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `vidu-q3-reference`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputViduQ3Reference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Output aspect ratio.
    pub aspect_ratio: InputViduQ3ReferenceAspectRatio,
    /// Output resolution.
    pub resolution: InputViduQ3ReferenceResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Reference images.
    #[serde(default)]
    pub images: Vec<InputViduQ3ReferenceImagesItem>,
}

impl InputViduQ3Reference {
    pub fn builder() -> InputViduQ3ReferenceBuilder {
        <InputViduQ3ReferenceBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputViduQ3ReferenceBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    seed: Option<i64>,
    aspect_ratio: Option<InputViduQ3ReferenceAspectRatio>,
    resolution: Option<InputViduQ3ReferenceResolution>,
    duration_ms: Option<i64>,
    images: Option<Vec<InputViduQ3ReferenceImagesItem>>,
}

impl InputViduQ3ReferenceBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputViduQ3ReferenceAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputViduQ3ReferenceResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputViduQ3ReferenceImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputViduQ3Reference`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputViduQ3ReferenceBuilder::prompt)
    /// - [`aspect_ratio`](InputViduQ3ReferenceBuilder::aspect_ratio)
    /// - [`resolution`](InputViduQ3ReferenceBuilder::resolution)
    /// - [`duration_ms`](InputViduQ3ReferenceBuilder::duration_ms)
    /// - [`images`](InputViduQ3ReferenceBuilder::images)
    pub fn build(self) -> Result<InputViduQ3Reference, BuildError> {
        Ok(InputViduQ3Reference {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            seed: self.seed,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            images: self.images.ok_or_else(|| BuildError::missing_field("images"))?,
        })
    }
}
