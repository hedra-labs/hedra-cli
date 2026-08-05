pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One customer-visible lifecycle event for a v3 job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobLogItem {
    /// Monotonically increasing id of this event within the job's log. Cursors are separate opaque values (`next_cursor` / `logs_next_cursor`); do not send this id as one.
    #[serde(default)]
    pub id: i64,
    /// ISO-8601 instant the event was recorded.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub timestamp: DateTime<FixedOffset>,
    pub level: JobLogLevel,
    pub event: JobLogEvent,
    /// Human-readable summary of the event.
    #[serde(default)]
    pub message: String,
    pub source: JobLogSource,
    /// Structured detail specific to this event type; empty when the event carries none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, serde_json::Value>>,
}

impl JobLogItem {
    pub fn builder() -> JobLogItemBuilder {
        <JobLogItemBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct JobLogItemBuilder {
    id: Option<i64>,
    timestamp: Option<DateTime<FixedOffset>>,
    level: Option<JobLogLevel>,
    event: Option<JobLogEvent>,
    message: Option<String>,
    source: Option<JobLogSource>,
    data: Option<HashMap<String, serde_json::Value>>,
}

impl JobLogItemBuilder {
    pub fn id(mut self, value: i64) -> Self {
        self.id = Some(value);
        self
    }

    pub fn timestamp(mut self, value: DateTime<FixedOffset>) -> Self {
        self.timestamp = Some(value);
        self
    }

    pub fn level(mut self, value: JobLogLevel) -> Self {
        self.level = Some(value);
        self
    }

    pub fn event(mut self, value: JobLogEvent) -> Self {
        self.event = Some(value);
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn source(mut self, value: JobLogSource) -> Self {
        self.source = Some(value);
        self
    }

    pub fn data(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.data = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`JobLogItem`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](JobLogItemBuilder::id)
    /// - [`timestamp`](JobLogItemBuilder::timestamp)
    /// - [`level`](JobLogItemBuilder::level)
    /// - [`event`](JobLogItemBuilder::event)
    /// - [`message`](JobLogItemBuilder::message)
    /// - [`source`](JobLogItemBuilder::source)
    pub fn build(self) -> Result<JobLogItem, BuildError> {
        Ok(JobLogItem {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            timestamp: self.timestamp.ok_or_else(|| BuildError::missing_field("timestamp"))?,
            level: self.level.ok_or_else(|| BuildError::missing_field("level"))?,
            event: self.event.ok_or_else(|| BuildError::missing_field("event"))?,
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
            source: self.source.ok_or_else(|| BuildError::missing_field("source"))?,
            data: self.data,
        })
    }
}
