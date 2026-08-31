pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-o1`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, prompt, start_image
/// (2) requires: aspect_ratio, duration_ms, images, prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingO1 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputKlingO1AspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKlingO1Resolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputKlingO1StartImage>,
    /// End frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputKlingO1EndImage>,
    /// Reference images. 1 to 3 images, each at most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputKlingO1ImagesItem>>,
}

impl InputKlingO1 {
    pub fn builder() -> InputKlingO1Builder {
        <InputKlingO1Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingO1Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputKlingO1AspectRatio>,
    resolution: Option<InputKlingO1Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputKlingO1StartImage>,
    end_image: Option<InputKlingO1EndImage>,
    images: Option<Vec<InputKlingO1ImagesItem>>,
}

impl InputKlingO1Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputKlingO1AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingO1Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKlingO1StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputKlingO1EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputKlingO1ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingO1`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKlingO1Builder::prompt)
    /// - [`aspect_ratio`](InputKlingO1Builder::aspect_ratio)
    /// - [`duration_ms`](InputKlingO1Builder::duration_ms)
    pub fn build(self) -> Result<InputKlingO1, BuildError> {
        Ok(InputKlingO1 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            images: self.images,
        })
    }
}
