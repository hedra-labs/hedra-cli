pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitResponse {
    /// This job's id — server-issued, and opaque.
    #[serde(default)]
    pub job_id: String,
    /// The resolved model id this job runs on.
    #[serde(default)]
    pub model: String,
    pub status: JobStatus,
    /// Path of this job's status monitor: poll GET /v3/jobs/{job_id}/status for status, progress, and an estimate.
    #[serde(default)]
    pub status_url: String,
    /// Path of the job resource itself: GET /v3/jobs/{job_id} returns the result envelope, including the outputs once it completes. Also the value of this response's `Location` header.
    #[serde(default)]
    pub result_url: String,
    /// ISO-8601 instant this job is estimated to finish. Null when no estimate exists for the model yet; poll GET /v3/jobs/{job_id}/status for a refreshed one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_at: Option<DateTime<FixedOffset>>,
}

impl SubmitResponse {
    pub fn builder() -> SubmitResponseBuilder {
        <SubmitResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitResponseBuilder {
    job_id: Option<String>,
    model: Option<String>,
    status: Option<JobStatus>,
    status_url: Option<String>,
    result_url: Option<String>,
    estimated_completion_at: Option<DateTime<FixedOffset>>,
}

impl SubmitResponseBuilder {
    pub fn job_id(mut self, value: impl Into<String>) -> Self {
        self.job_id = Some(value.into());
        self
    }

    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn status(mut self, value: JobStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn status_url(mut self, value: impl Into<String>) -> Self {
        self.status_url = Some(value.into());
        self
    }

    pub fn result_url(mut self, value: impl Into<String>) -> Self {
        self.result_url = Some(value.into());
        self
    }

    pub fn estimated_completion_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.estimated_completion_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SubmitResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`job_id`](SubmitResponseBuilder::job_id)
    /// - [`model`](SubmitResponseBuilder::model)
    /// - [`status`](SubmitResponseBuilder::status)
    /// - [`status_url`](SubmitResponseBuilder::status_url)
    /// - [`result_url`](SubmitResponseBuilder::result_url)
    pub fn build(self) -> Result<SubmitResponse, BuildError> {
        Ok(SubmitResponse {
            job_id: self.job_id.ok_or_else(|| BuildError::missing_field("job_id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            status_url: self.status_url.ok_or_else(|| BuildError::missing_field("status_url"))?,
            result_url: self.result_url.ok_or_else(|| BuildError::missing_field("result_url"))?,
            estimated_completion_at: self.estimated_completion_at,
        })
    }
}
