pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WebhookDeliverySummary {
    #[serde(default)]
    pub job_id: String,
    #[serde(default)]
    pub job_accessible: bool,
    #[serde(default)]
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<WebhookEventType>,
    pub status: WebhookDeliveryStatus,
    pub source: WebhookDeliverySource,
    #[serde(default)]
    pub attempts: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redelivery_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeliveries: Option<Vec<WebhookRedelivery>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_response_status: Option<i64>,
    /// Why the most recent delivery attempt failed, in the same error envelope `GET /jobs/{job_id}` returns for a failed job: a stable `code` from the shared error vocabulary, a fixed operator-facing `message`, and `retryable`. Null while no attempt has failed. Destination URLs, addresses, headers, credentials, response bodies, and internal exception text are never included — those stay in Hedra's own logs. `retryable` describes the condition, not what Hedra did: every non-2xx response is retried on the published ladder, so it answers whether replaying this delivery is likely to help. Deliveries that failed before this field became structured report `UNKNOWN`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<ErrorEnvelope>,
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<DateTime<FixedOffset>>,
}

impl WebhookDeliverySummary {
    pub fn builder() -> WebhookDeliverySummaryBuilder {
        <WebhookDeliverySummaryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookDeliverySummaryBuilder {
    job_id: Option<String>,
    job_accessible: Option<bool>,
    model: Option<String>,
    event_type: Option<WebhookEventType>,
    status: Option<WebhookDeliveryStatus>,
    source: Option<WebhookDeliverySource>,
    attempts: Option<i64>,
    redelivery_count: Option<i64>,
    redeliveries: Option<Vec<WebhookRedelivery>>,
    last_response_status: Option<i64>,
    last_error: Option<ErrorEnvelope>,
    webhook_url: Option<String>,
    created_at: Option<DateTime<FixedOffset>>,
    last_attempt_at: Option<DateTime<FixedOffset>>,
}

impl WebhookDeliverySummaryBuilder {
    pub fn job_id(mut self, value: impl Into<String>) -> Self {
        self.job_id = Some(value.into());
        self
    }

    pub fn job_accessible(mut self, value: bool) -> Self {
        self.job_accessible = Some(value);
        self
    }

    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn event_type(mut self, value: WebhookEventType) -> Self {
        self.event_type = Some(value);
        self
    }

    pub fn status(mut self, value: WebhookDeliveryStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn source(mut self, value: WebhookDeliverySource) -> Self {
        self.source = Some(value);
        self
    }

    pub fn attempts(mut self, value: i64) -> Self {
        self.attempts = Some(value);
        self
    }

    pub fn redelivery_count(mut self, value: i64) -> Self {
        self.redelivery_count = Some(value);
        self
    }

    pub fn redeliveries(mut self, value: Vec<WebhookRedelivery>) -> Self {
        self.redeliveries = Some(value);
        self
    }

    pub fn last_response_status(mut self, value: i64) -> Self {
        self.last_response_status = Some(value);
        self
    }

    pub fn last_error(mut self, value: ErrorEnvelope) -> Self {
        self.last_error = Some(value);
        self
    }

    pub fn webhook_url(mut self, value: impl Into<String>) -> Self {
        self.webhook_url = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn last_attempt_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.last_attempt_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`WebhookDeliverySummary`].
    /// This method will fail if any of the following fields are not set:
    /// - [`job_id`](WebhookDeliverySummaryBuilder::job_id)
    /// - [`job_accessible`](WebhookDeliverySummaryBuilder::job_accessible)
    /// - [`model`](WebhookDeliverySummaryBuilder::model)
    /// - [`status`](WebhookDeliverySummaryBuilder::status)
    /// - [`source`](WebhookDeliverySummaryBuilder::source)
    /// - [`attempts`](WebhookDeliverySummaryBuilder::attempts)
    /// - [`webhook_url`](WebhookDeliverySummaryBuilder::webhook_url)
    /// - [`created_at`](WebhookDeliverySummaryBuilder::created_at)
    pub fn build(self) -> Result<WebhookDeliverySummary, BuildError> {
        Ok(WebhookDeliverySummary {
            job_id: self.job_id.ok_or_else(|| BuildError::missing_field("job_id"))?,
            job_accessible: self.job_accessible.ok_or_else(|| BuildError::missing_field("job_accessible"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            event_type: self.event_type,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            source: self.source.ok_or_else(|| BuildError::missing_field("source"))?,
            attempts: self.attempts.ok_or_else(|| BuildError::missing_field("attempts"))?,
            redelivery_count: self.redelivery_count,
            redeliveries: self.redeliveries,
            last_response_status: self.last_response_status,
            last_error: self.last_error,
            webhook_url: self.webhook_url.ok_or_else(|| BuildError::missing_field("webhook_url"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            last_attempt_at: self.last_attempt_at,
        })
    }
}
