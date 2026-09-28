pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A job's status, progress, and estimated completion time.
/// 
/// Returned by `GET /v3/jobs/{job_id}/status`, and sent as each `status` frame
/// of `GET /v3/jobs/{job_id}/stream`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusResponse {
    /// The job this status describes.
    #[serde(default)]
    pub job_id: String,
    pub status: JobStatus,
    /// Estimated fraction of the job completed, from 0 to 1 (not a percentage). While `estimated_completion_at` is non-null, this value is computed from the time since submission and that estimate, so it is only as accurate as the estimate, and it stays below 1. While an `IN_QUEUE` or `IN_PROGRESS` job's `estimated_completion_at` is null, `GET /v3/jobs/{job_id}/status` reports 0. A `COMPLETED` job reports 1. A `FAILED` job's value can be anything from 0 to 1 and does not show how far the job got.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    /// ISO-8601 instant this job is estimated to finish. A job can finish well before or after it. The estimate can change while the job runs, so read it from each poll. Null once the job is `COMPLETED` or `FAILED`, and while no estimate is available. The submit response also includes it; `GET /v3/jobs/{job_id}` and `GET /v3/jobs` do not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_at: Option<DateTime<FixedOffset>>,
    /// Lifecycle events newer than the `logs_after` cursor, oldest first. Present only when `logs_after` is supplied; absent from the stream's `status` frames.
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
