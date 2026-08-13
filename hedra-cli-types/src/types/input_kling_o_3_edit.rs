pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-o3-edit`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingO3Edit {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(default)]
    pub prompt: String,
    /// Source video (video-to-video). From 3s to 15s and at most 524.2 MB.
    pub source_video: InputKlingO3EditSourceVideo,
    /// Reference images. 1 to 4 images, each at least 300px on each side and at most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputKlingO3EditImagesItem>>,
    /// Output resolution to generate at.
    pub resolution: InputKlingO3EditResolution,
}

impl InputKlingO3Edit {
    pub fn builder() -> InputKlingO3EditBuilder {
        <InputKlingO3EditBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingO3EditBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    source_video: Option<InputKlingO3EditSourceVideo>,
    images: Option<Vec<InputKlingO3EditImagesItem>>,
    resolution: Option<InputKlingO3EditResolution>,
}

impl InputKlingO3EditBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn source_video(mut self, value: InputKlingO3EditSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputKlingO3EditImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingO3EditResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingO3Edit`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKlingO3EditBuilder::prompt)
    /// - [`source_video`](InputKlingO3EditBuilder::source_video)
    /// - [`resolution`](InputKlingO3EditBuilder::resolution)
    pub fn build(self) -> Result<InputKlingO3Edit, BuildError> {
        Ok(InputKlingO3Edit {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            images: self.images,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
        })
    }
}
