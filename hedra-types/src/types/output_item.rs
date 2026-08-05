pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One item of a job's `outputs[]` (`GET /v3/jobs/{job_id}`). `outputs` is
/// always an array, even for a single output.
/// 
/// Every key is always present. The ones a modality carries no value for
/// serialize as null — an image output reports `duration_ms: null` and
/// `fps: null`, an audio output `width: null` — so the shape is one object
/// rather than one per modality. `url` and `asset_id` go null on an expired
/// output, which has metadata but no retrievable bytes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OutputItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OutputStatus>,
    /// This output's asset id — server-issued, and opaque. Pass it as `{"source": "asset", "asset_id": ...}` in a later submit's media inputs to reuse this output as a reference. Null once the output has expired, since its bytes are no longer retrievable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// Presigned download URL for the output bytes. Null once the output has expired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// MIME type of the output bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Width in pixels; null for an output with no frame (audio).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    /// Height in pixels; null for an output with no frame (audio).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    /// Duration in milliseconds; null for a still image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// The video's measured frame rate; null for a non-video output and for a video that has not been probed yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<i64>,
    /// Why this item failed — present on a failed item within an otherwise-completed batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,
}

impl OutputItem {
    pub fn builder() -> OutputItemBuilder {
        <OutputItemBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct OutputItemBuilder {
    status: Option<OutputStatus>,
    asset_id: Option<String>,
    url: Option<String>,
    content_type: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    duration_ms: Option<i64>,
    fps: Option<i64>,
    error: Option<ErrorEnvelope>,
}

impl OutputItemBuilder {
    pub fn status(mut self, value: OutputStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn asset_id(mut self, value: impl Into<String>) -> Self {
        self.asset_id = Some(value.into());
        self
    }

    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn content_type(mut self, value: impl Into<String>) -> Self {
        self.content_type = Some(value.into());
        self
    }

    pub fn width(mut self, value: i64) -> Self {
        self.width = Some(value);
        self
    }

    pub fn height(mut self, value: i64) -> Self {
        self.height = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn fps(mut self, value: i64) -> Self {
        self.fps = Some(value);
        self
    }

    pub fn error(mut self, value: ErrorEnvelope) -> Self {
        self.error = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`OutputItem`].
    pub fn build(self) -> Result<OutputItem, BuildError> {
        Ok(OutputItem {
            status: self.status,
            asset_id: self.asset_id,
            url: self.url,
            content_type: self.content_type,
            width: self.width,
            height: self.height,
            duration_ms: self.duration_ms,
            fps: self.fps,
            error: self.error,
        })
    }
}
