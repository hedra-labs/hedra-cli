pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-o3-reference`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingO3Reference {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(default)]
    pub prompt: String,
    /// Source video (video-to-video). From 3s to 15s and at most 524.2 MB.
    pub source_video: InputKlingO3ReferenceSourceVideo,
    /// Reference images. 1 to 4 images, each at least 300px on each side and at most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputKlingO3ReferenceImagesItem>>,
    /// Output resolution to generate at.
    pub resolution: InputKlingO3ReferenceResolution,
}

impl InputKlingO3Reference {
    pub fn builder() -> InputKlingO3ReferenceBuilder {
        <InputKlingO3ReferenceBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingO3ReferenceBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    source_video: Option<InputKlingO3ReferenceSourceVideo>,
    images: Option<Vec<InputKlingO3ReferenceImagesItem>>,
    resolution: Option<InputKlingO3ReferenceResolution>,
}

impl InputKlingO3ReferenceBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn source_video(mut self, value: InputKlingO3ReferenceSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputKlingO3ReferenceImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingO3ReferenceResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingO3Reference`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKlingO3ReferenceBuilder::prompt)
    /// - [`source_video`](InputKlingO3ReferenceBuilder::source_video)
    /// - [`resolution`](InputKlingO3ReferenceBuilder::resolution)
    pub fn build(self) -> Result<InputKlingO3Reference, BuildError> {
        Ok(InputKlingO3Reference {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            images: self.images,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
        })
    }
}
