pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `veed-video-background-removal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputVeedVideoBackgroundRemoval {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Video whose background should be made transparent. At most 120s and at most 524.2 MB.
    pub source_video: InputVeedVideoBackgroundRemovalSourceVideo,
    /// Optimize segmentation for a person rather than an object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_is_person: Option<bool>,
}

impl InputVeedVideoBackgroundRemoval {
    pub fn builder() -> InputVeedVideoBackgroundRemovalBuilder {
        <InputVeedVideoBackgroundRemovalBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputVeedVideoBackgroundRemovalBuilder {
    num_outputs: Option<i64>,
    source_video: Option<InputVeedVideoBackgroundRemovalSourceVideo>,
    subject_is_person: Option<bool>,
}

impl InputVeedVideoBackgroundRemovalBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputVeedVideoBackgroundRemovalSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn subject_is_person(mut self, value: bool) -> Self {
        self.subject_is_person = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputVeedVideoBackgroundRemoval`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_video`](InputVeedVideoBackgroundRemovalBuilder::source_video)
    pub fn build(self) -> Result<InputVeedVideoBackgroundRemoval, BuildError> {
        Ok(InputVeedVideoBackgroundRemoval {
            num_outputs: self.num_outputs,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            subject_is_person: self.subject_is_person,
        })
    }
}
