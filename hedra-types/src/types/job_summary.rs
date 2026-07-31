pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JobSummary {
    #[serde(default)]
    pub job_id: String,
    #[serde(default)]
    pub model: String,
    /// The quality level this job ran at; present only for models that offer quality levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    pub status: JobStatus,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
}

impl JobSummary {
    pub fn builder() -> JobSummaryBuilder {
        <JobSummaryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct JobSummaryBuilder {
    job_id: Option<String>,
    model: Option<String>,
    quality: Option<String>,
    status: Option<JobStatus>,
    created_at: Option<DateTime<FixedOffset>>,
}

impl JobSummaryBuilder {
    pub fn job_id(mut self, value: impl Into<String>) -> Self {
        self.job_id = Some(value.into());
        self
    }

    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn quality(mut self, value: impl Into<String>) -> Self {
        self.quality = Some(value.into());
        self
    }

    pub fn status(mut self, value: JobStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`JobSummary`].
    /// This method will fail if any of the following fields are not set:
    /// - [`job_id`](JobSummaryBuilder::job_id)
    /// - [`model`](JobSummaryBuilder::model)
    /// - [`status`](JobSummaryBuilder::status)
    /// - [`created_at`](JobSummaryBuilder::created_at)
    pub fn build(self) -> Result<JobSummary, BuildError> {
        Ok(JobSummary {
            job_id: self.job_id.ok_or_else(|| BuildError::missing_field("job_id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            quality: self.quality,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
        })
    }
}
