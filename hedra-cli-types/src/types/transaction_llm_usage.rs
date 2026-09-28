pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The chat requests an `llm_usage` transaction sums.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct TransactionLlmUsage {
    /// The model the requests used.
    #[serde(default)]
    pub model: String,
    /// How many requests the row sums.
    #[serde(default)]
    pub request_count: i64,
    /// Start of the UTC day the row covers. A request counts toward the day it was created, even when it was charged after midnight. `GET /v3/usage/llm` lists your own requests in the row when given this as `start`, `period_end` as `end`, and `model`.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub period_start: DateTime<FixedOffset>,
    /// End of the UTC day the row covers, exclusive: one day after `period_start`.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub period_end: DateTime<FixedOffset>,
}

impl TransactionLlmUsage {
    pub fn builder() -> TransactionLlmUsageBuilder {
        <TransactionLlmUsageBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct TransactionLlmUsageBuilder {
    model: Option<String>,
    request_count: Option<i64>,
    period_start: Option<DateTime<FixedOffset>>,
    period_end: Option<DateTime<FixedOffset>>,
}

impl TransactionLlmUsageBuilder {
    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn request_count(mut self, value: i64) -> Self {
        self.request_count = Some(value);
        self
    }

    pub fn period_start(mut self, value: DateTime<FixedOffset>) -> Self {
        self.period_start = Some(value);
        self
    }

    pub fn period_end(mut self, value: DateTime<FixedOffset>) -> Self {
        self.period_end = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`TransactionLlmUsage`].
    /// This method will fail if any of the following fields are not set:
    /// - [`model`](TransactionLlmUsageBuilder::model)
    /// - [`request_count`](TransactionLlmUsageBuilder::request_count)
    /// - [`period_start`](TransactionLlmUsageBuilder::period_start)
    /// - [`period_end`](TransactionLlmUsageBuilder::period_end)
    pub fn build(self) -> Result<TransactionLlmUsage, BuildError> {
        Ok(TransactionLlmUsage {
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            request_count: self.request_count.ok_or_else(|| BuildError::missing_field("request_count"))?,
            period_start: self.period_start.ok_or_else(|| BuildError::missing_field("period_start"))?,
            period_end: self.period_end.ok_or_else(|| BuildError::missing_field("period_end"))?,
        })
    }
}
