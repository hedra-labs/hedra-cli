pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Body of a webhook delivery: the `GET /jobs/{job_id}` result envelope without its poll-only fields (`logs`, `cost`, `currency`) — poll the job to read those.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WebhookPayload {
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
}

impl WebhookPayload {
    pub fn builder() -> WebhookPayloadBuilder {
        <WebhookPayloadBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookPayloadBuilder {
    job_id: Option<String>,
    model: Option<String>,
    quality: Option<String>,
    status: Option<JobStatus>,
    prompt: Option<String>,
    outputs: Option<Vec<OutputItem>>,
    metrics: Option<Metrics>,
    error: Option<ErrorEnvelope>,
}

impl WebhookPayloadBuilder {
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

    /// Consumes the builder and constructs a [`WebhookPayload`].
    /// This method will fail if any of the following fields are not set:
    /// - [`job_id`](WebhookPayloadBuilder::job_id)
    /// - [`model`](WebhookPayloadBuilder::model)
    /// - [`status`](WebhookPayloadBuilder::status)
    pub fn build(self) -> Result<WebhookPayload, BuildError> {
        Ok(WebhookPayload {
            job_id: self.job_id.ok_or_else(|| BuildError::missing_field("job_id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            quality: self.quality,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            prompt: self.prompt,
            outputs: self.outputs,
            metrics: self.metrics,
            error: self.error,
        })
    }
}
