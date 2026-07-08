pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// ``POST /v3/files`` result. The presigned ``url`` *is* the file handle:
/// pass it as ``input.image_url`` / ``audio_url`` / ``video_url`` on submit
/// (once reference inputs are wired — see the v3 plan doc).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct FileUploadResponse {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub expires_at: String,
}

impl FileUploadResponse {
    pub fn builder() -> FileUploadResponseBuilder {
        <FileUploadResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct FileUploadResponseBuilder {
    url: Option<String>,
    content_type: Option<String>,
    expires_at: Option<String>,
}

impl FileUploadResponseBuilder {
    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn content_type(mut self, value: impl Into<String>) -> Self {
        self.content_type = Some(value.into());
        self
    }

    pub fn expires_at(mut self, value: impl Into<String>) -> Self {
        self.expires_at = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`FileUploadResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`url`](FileUploadResponseBuilder::url)
    /// - [`content_type`](FileUploadResponseBuilder::content_type)
    /// - [`expires_at`](FileUploadResponseBuilder::expires_at)
    pub fn build(self) -> Result<FileUploadResponse, BuildError> {
        Ok(FileUploadResponse {
            url: self.url.ok_or_else(|| BuildError::missing_field("url"))?,
            content_type: self.content_type.ok_or_else(|| BuildError::missing_field("content_type"))?,
            expires_at: self.expires_at.ok_or_else(|| BuildError::missing_field("expires_at"))?,
        })
    }
}
