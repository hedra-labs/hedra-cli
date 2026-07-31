pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The lightweight poll envelope, also the SSE ``status`` frame and the MCP
/// progress notification payload.
/// 
/// The two log fields are poll-only by construction, like ``cost`` on
/// the result envelope: they are populated when — and only when — a
/// ``GET /v3/jobs/{job_id}/status`` caller supplies a ``logs_after`` cursor, and
/// they exclude when ``None``, so the frame the stream transports build from a
/// Redis payload with no DB session is byte-identical to a log-free envelope.
/// That was ENG-9693's reason for having no ``logs`` field at all; the stream
/// now carries lifecycle rows as their own ``event: log`` frames instead of
/// inside this one (ENG-9694), and MCP progress notifications stay status-only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusResponse {
    #[serde(default)]
    pub job_id: String,
    pub status: JobStatus,
    /// Fraction of the job completed, from 0 to 1 (not a percentage). Null when the job has not reported progress yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_at: Option<DateTime<FixedOffset>>,
    /// Lifecycle events newer than the `logs_after` cursor, oldest first. Present only when `logs_after` is supplied; absent from SSE status frames and MCP progress notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobLogItem>>,
    /// Cursor to send as `logs_after` on the next poll. Absent when this poll delivered no new events — keep using the cursor you sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_next_cursor: Option<String>,
}

impl StatusResponse {
    pub fn builder() -> StatusResponseBuilder {
        <StatusResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct StatusResponseBuilder {
    job_id: Option<String>,
    status: Option<JobStatus>,
    progress: Option<f64>,
    estimated_completion_at: Option<DateTime<FixedOffset>>,
    logs: Option<Vec<JobLogItem>>,
    logs_next_cursor: Option<String>,
}

impl StatusResponseBuilder {
    pub fn job_id(mut self, value: impl Into<String>) -> Self {
        self.job_id = Some(value.into());
        self
    }

    pub fn status(mut self, value: JobStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn progress(mut self, value: f64) -> Self {
        self.progress = Some(value);
        self
    }

    pub fn estimated_completion_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.estimated_completion_at = Some(value);
        self
    }

    pub fn logs(mut self, value: Vec<JobLogItem>) -> Self {
        self.logs = Some(value);
        self
    }

    pub fn logs_next_cursor(mut self, value: impl Into<String>) -> Self {
        self.logs_next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`StatusResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`job_id`](StatusResponseBuilder::job_id)
    /// - [`status`](StatusResponseBuilder::status)
    pub fn build(self) -> Result<StatusResponse, BuildError> {
        Ok(StatusResponse {
            job_id: self.job_id.ok_or_else(|| BuildError::missing_field("job_id"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            progress: self.progress,
            estimated_completion_at: self.estimated_completion_at,
            logs: self.logs,
            logs_next_cursor: self.logs_next_cursor,
        })
    }
}
