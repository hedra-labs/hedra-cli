pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The outcome a replay superseded: the delivery's own fields as they stood
/// when the operator requested the redelivery. Only finished deliveries can be
/// replayed, so `status` is always terminal (`DELIVERED` / `FAILED`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WebhookRedelivery {
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub requested_at: DateTime<FixedOffset>,
    pub status: WebhookDeliveryStatus,
    #[serde(default)]
    pub attempts: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_response_status: Option<i64>,
    /// Why the most recent delivery attempt failed, in the same error envelope `GET /jobs/{job_id}` returns for a failed job: a stable `code` from the shared error vocabulary, a fixed operator-facing `message`, and `retryable`. Null while no attempt has failed. Destination URLs, addresses, headers, credentials, response bodies, and internal exception text are never included — those stay in Hedra's own logs. `retryable` describes the condition, not what Hedra did: every non-2xx response is retried on the published ladder, so it answers whether replaying this delivery is likely to help. Deliveries that failed before this field became structured report `UNKNOWN`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<ErrorEnvelope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<DateTime<FixedOffset>>,
}

impl WebhookRedelivery {
    pub fn builder() -> WebhookRedeliveryBuilder {
        <WebhookRedeliveryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookRedeliveryBuilder {
    requested_at: Option<DateTime<FixedOffset>>,
    status: Option<WebhookDeliveryStatus>,
    attempts: Option<i64>,
    last_response_status: Option<i64>,
    last_error: Option<ErrorEnvelope>,
    last_attempt_at: Option<DateTime<FixedOffset>>,
}

impl WebhookRedeliveryBuilder {
    pub fn requested_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.requested_at = Some(value);
        self
    }

    pub fn status(mut self, value: WebhookDeliveryStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn attempts(mut self, value: i64) -> Self {
        self.attempts = Some(value);
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

    pub fn last_attempt_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.last_attempt_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`WebhookRedelivery`].
    /// This method will fail if any of the following fields are not set:
    /// - [`requested_at`](WebhookRedeliveryBuilder::requested_at)
    /// - [`status`](WebhookRedeliveryBuilder::status)
    /// - [`attempts`](WebhookRedeliveryBuilder::attempts)
    pub fn build(self) -> Result<WebhookRedelivery, BuildError> {
        Ok(WebhookRedelivery {
            requested_at: self.requested_at.ok_or_else(|| BuildError::missing_field("requested_at"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            attempts: self.attempts.ok_or_else(|| BuildError::missing_field("attempts"))?,
            last_response_status: self.last_response_status,
            last_error: self.last_error,
            last_attempt_at: self.last_attempt_at,
        })
    }
}
