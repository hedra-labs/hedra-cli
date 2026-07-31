pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultResponse {
    #[serde(default)]
    pub job_id: String,
    #[serde(default)]
    pub model: String,
    /// The quality level this job ran at; present only for models that offer quality levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    pub status: JobStatus,
    /// The prompt this job ran with. When `enhance_prompt` was set, this is the rewritten prompt the model received rather than the one submitted. Absent on models that take no prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<OutputItem>>,
    /// Timing for this job; present on completed jobs only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Metrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,
    /// The most recent lifecycle events for this job, oldest first. Capped; GET /v3/jobs/{job_id}/logs serves the full paginated history. Absent from webhook payloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<JobLogItem>>,
    /// Net cost of this job; 0 when fully refunded; absent until charged. Absent from webhook payloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    /// ISO-4217 currency code for `cost`. Present exactly when `cost` is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

impl ResultResponse {
    pub fn builder() -> ResultResponseBuilder {
        <ResultResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ResultResponseBuilder {
    job_id: Option<String>,
    model: Option<String>,
    quality: Option<String>,
    status: Option<JobStatus>,
    prompt: Option<String>,
    outputs: Option<Vec<OutputItem>>,
    metrics: Option<Metrics>,
    error: Option<ErrorEnvelope>,
    logs: Option<Vec<JobLogItem>>,
    cost: Option<f64>,
    currency: Option<String>,
}

impl ResultResponseBuilder {
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

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn outputs(mut self, value: Vec<OutputItem>) -> Self {
        self.outputs = Some(value);
        self
    }

    pub fn metrics(mut self, value: Metrics) -> Self {
        self.metrics = Some(value);
        self
    }

    pub fn error(mut self, value: ErrorEnvelope) -> Self {
        self.error = Some(value);
        self
    }

    pub fn logs(mut self, value: Vec<JobLogItem>) -> Self {
        self.logs = Some(value);
        self
    }

    pub fn cost(mut self, value: f64) -> Self {
        self.cost = Some(value);
        self
    }

    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ResultResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`job_id`](ResultResponseBuilder::job_id)
    /// - [`model`](ResultResponseBuilder::model)
    /// - [`status`](ResultResponseBuilder::status)
    pub fn build(self) -> Result<ResultResponse, BuildError> {
        Ok(ResultResponse {
            job_id: self.job_id.ok_or_else(|| BuildError::missing_field("job_id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            quality: self.quality,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            prompt: self.prompt,
            outputs: self.outputs,
            metrics: self.metrics,
            error: self.error,
            logs: self.logs,
            cost: self.cost,
            currency: self.currency,
        })
    }
}
