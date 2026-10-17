pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `krea-3`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKrea3 {
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Not available on this model. Its price is computed from the exact prompt submitted, which a rewrite would invalidate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Images to edit or blend. 1 to 4 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputKrea3ImagesItem>>,
    /// Output aspect ratio.
    pub aspect_ratio: InputKrea3AspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKrea3Resolution>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl InputKrea3 {
    pub fn builder() -> InputKrea3Builder {
        <InputKrea3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKrea3Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    images: Option<Vec<InputKrea3ImagesItem>>,
    aspect_ratio: Option<InputKrea3AspectRatio>,
    resolution: Option<InputKrea3Resolution>,
    seed: Option<i64>,
}

impl InputKrea3Builder {
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

    pub fn images(mut self, value: Vec<InputKrea3ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputKrea3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKrea3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKrea3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKrea3Builder::prompt)
    /// - [`aspect_ratio`](InputKrea3Builder::aspect_ratio)
    pub fn build(self) -> Result<InputKrea3, BuildError> {
        Ok(InputKrea3 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            images: self.images,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            seed: self.seed,
        })
    }
}
