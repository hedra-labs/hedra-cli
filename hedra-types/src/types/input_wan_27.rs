pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `wan-2-7`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, images, prompt, resolution
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution
/// (3) requires: duration_ms, prompt, resolution, start_image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputWan27 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// What to avoid in the generated video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Seed for reproducible output; omit for a random seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Output aspect ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<InputWan27AspectRatio>,
    /// Output resolution.
    pub resolution: InputWan27Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Reference images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputWan27ImagesItem>>,
    /// Start frame (image-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputWan27StartImage>,
    /// End frame (first-last-frame-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputWan27EndImage>,
}

impl InputWan27 {
    pub fn builder() -> InputWan27Builder {
        <InputWan27Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputWan27Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    negative_prompt: Option<String>,
    seed: Option<i64>,
    aspect_ratio: Option<InputWan27AspectRatio>,
    resolution: Option<InputWan27Resolution>,
    duration_ms: Option<i64>,
    images: Option<Vec<InputWan27ImagesItem>>,
    start_image: Option<InputWan27StartImage>,
    end_image: Option<InputWan27EndImage>,
}

impl InputWan27Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn negative_prompt(mut self, value: impl Into<String>) -> Self {
        self.negative_prompt = Some(value.into());
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputWan27AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputWan27Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputWan27ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputWan27StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputWan27EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputWan27`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputWan27Builder::prompt)
    /// - [`resolution`](InputWan27Builder::resolution)
    /// - [`duration_ms`](InputWan27Builder::duration_ms)
    pub fn build(self) -> Result<InputWan27, BuildError> {
        Ok(InputWan27 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            negative_prompt: self.negative_prompt,
            seed: self.seed,
            aspect_ratio: self.aspect_ratio,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            images: self.images,
            start_image: self.start_image,
            end_image: self.end_image,
        })
    }
}
