pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One chat-completion request's metering record.
/// 
/// Metadata only: prompt and completion content are never retained, so there
/// is no content to expose — by construction, not by filtering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmUsageRecord {
    /// The request's id — the same value the chat response returned in its `X-Request-Id` header.
    #[serde(default)]
    pub id: String,
    /// The model id the request was admitted for.
    #[serde(default)]
    pub model: String,
    /// `pending` while the request is in flight, `ok` once its usage settled, `upstream_error`/`client_abort`/`timeout` for requests that ended without a normal completion, and `settlement_failed` for delivered work whose usage could not be recorded in time — a later recovery may still fill in the amount.
    pub status: LlmUsageStatus,
    /// Whether the response was streamed.
    #[serde(default)]
    pub streamed: bool,
    /// Prompt tokens the request consumed; null until settled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<i64>,
    /// Completion tokens generated; null until settled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<i64>,
    /// Reasoning tokens, as a breakdown of `completion_tokens` (never an addend); null when the model reports none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<i64>,
    /// True when the token counts were estimated rather than reported by the model runtime.
    #[serde(default)]
    pub usage_estimated: bool,
    /// What this request cost; null until its usage settles.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub cost: Option<f64>,
    /// ISO-4217 currency code for `cost`. Present exactly when `cost` is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Milliseconds to first token; null when not measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttft_ms: Option<i64>,
    /// Total request duration in milliseconds; null while in flight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// `key_id` of the API key that made the request. Null for requests not attributable to a current key: session-token requests, requests whose key was since deleted, and legacy keys that predate public key ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    /// The completion id (`id` in the chat response body), for correlating a record to a completion; null when the request did not yield a recordable one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_id: Option<String>,
    /// ISO-8601 instant the request was admitted.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    /// ISO-8601 instant the usage settled; null until then.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub settled_at: Option<DateTime<FixedOffset>>,
}

impl LlmUsageRecord {
    pub fn builder() -> LlmUsageRecordBuilder {
        <LlmUsageRecordBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LlmUsageRecordBuilder {
    id: Option<String>,
    model: Option<String>,
    status: Option<LlmUsageStatus>,
    streamed: Option<bool>,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    reasoning_tokens: Option<i64>,
    usage_estimated: Option<bool>,
    cost: Option<f64>,
    currency: Option<String>,
    ttft_ms: Option<i64>,
    duration_ms: Option<i64>,
    key_id: Option<String>,
    completion_id: Option<String>,
    created_at: Option<DateTime<FixedOffset>>,
    settled_at: Option<DateTime<FixedOffset>>,
}

impl LlmUsageRecordBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn status(mut self, value: LlmUsageStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn streamed(mut self, value: bool) -> Self {
        self.streamed = Some(value);
        self
    }

    pub fn prompt_tokens(mut self, value: i64) -> Self {
        self.prompt_tokens = Some(value);
        self
    }

    pub fn completion_tokens(mut self, value: i64) -> Self {
        self.completion_tokens = Some(value);
        self
    }

    pub fn reasoning_tokens(mut self, value: i64) -> Self {
        self.reasoning_tokens = Some(value);
        self
    }

    pub fn usage_estimated(mut self, value: bool) -> Self {
        self.usage_estimated = Some(value);
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

    pub fn ttft_ms(mut self, value: i64) -> Self {
        self.ttft_ms = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn key_id(mut self, value: impl Into<String>) -> Self {
        self.key_id = Some(value.into());
        self
    }

    pub fn completion_id(mut self, value: impl Into<String>) -> Self {
        self.completion_id = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn settled_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.settled_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`LlmUsageRecord`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](LlmUsageRecordBuilder::id)
    /// - [`model`](LlmUsageRecordBuilder::model)
    /// - [`status`](LlmUsageRecordBuilder::status)
    /// - [`streamed`](LlmUsageRecordBuilder::streamed)
    /// - [`usage_estimated`](LlmUsageRecordBuilder::usage_estimated)
    /// - [`created_at`](LlmUsageRecordBuilder::created_at)
    pub fn build(self) -> Result<LlmUsageRecord, BuildError> {
        Ok(LlmUsageRecord {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            streamed: self.streamed.ok_or_else(|| BuildError::missing_field("streamed"))?,
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            reasoning_tokens: self.reasoning_tokens,
            usage_estimated: self.usage_estimated.ok_or_else(|| BuildError::missing_field("usage_estimated"))?,
            cost: self.cost,
            currency: self.currency,
            ttft_ms: self.ttft_ms,
            duration_ms: self.duration_ms,
            key_id: self.key_id,
            completion_id: self.completion_id,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            settled_at: self.settled_at,
        })
    }
}
