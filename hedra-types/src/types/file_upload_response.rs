pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// ``POST /v3/files`` result. The presigned ``url`` *is* the file handle:
/// pass it as ``{"source": "url", "url": <url>}`` in any media input the
/// model's schema advertises (``image`` / ``end_image`` / ``images`` /
/// ``audio`` / ``video``) on submit.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct FileUploadResponse {
    /// Presigned GET URL for the stored bytes — the handle itself. Pass it back verbatim, query string included: a modified URL is not recognised as your handle and is rejected as an external URL.
    #[serde(default)]
    pub url: String,
    /// MIME type sniffed from the uploaded bytes; the request's own Content-Type and filename are never trusted. Decides which media inputs will accept this handle.
    #[serde(default)]
    pub content_type: String,
    /// ISO-8601 instant `url` stops being accepted — one hour after upload. Submitting with a lapsed handle is a 400 with `reason: "expired"`, not a fetch failure; upload the file again for a fresh handle. The stored object is deleted separately by a bucket lifecycle policy about a day after upload, so treat re-uploading, not re-presigning, as the remedy.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub expires_at: DateTime<FixedOffset>,
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
    expires_at: Option<DateTime<FixedOffset>>,
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

    pub fn expires_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.expires_at = Some(value);
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
