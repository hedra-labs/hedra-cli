pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `eyeline-id-restyle`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputEyelineIdRestyle {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Source video. From 1s to 8s and at most 524.2 MB.
    pub source_video: InputEyelineIdRestyleSourceVideo,
    /// An edited copy of the source video's first frame. Exactly one; the clip is regenerated to match it. Exactly 1 image, at most 10.4 MB.
    #[serde(default)]
    pub images: Vec<InputEyelineIdRestyleImagesItem>,
}

impl InputEyelineIdRestyle {
    pub fn builder() -> InputEyelineIdRestyleBuilder {
        <InputEyelineIdRestyleBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputEyelineIdRestyleBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    source_video: Option<InputEyelineIdRestyleSourceVideo>,
    images: Option<Vec<InputEyelineIdRestyleImagesItem>>,
}

impl InputEyelineIdRestyleBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn source_video(mut self, value: InputEyelineIdRestyleSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputEyelineIdRestyleImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputEyelineIdRestyle`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputEyelineIdRestyleBuilder::prompt)
    /// - [`source_video`](InputEyelineIdRestyleBuilder::source_video)
    /// - [`images`](InputEyelineIdRestyleBuilder::images)
    pub fn build(self) -> Result<InputEyelineIdRestyle, BuildError> {
        Ok(InputEyelineIdRestyle {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            images: self.images.ok_or_else(|| BuildError::missing_field("images"))?,
        })
    }
}
