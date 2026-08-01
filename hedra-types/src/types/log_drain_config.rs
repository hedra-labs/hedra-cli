pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LogDrainConfig {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    pub format: LogDrainFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_names: Option<Vec<String>>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub batch_size: i64,
    #[serde(default)]
    pub consecutive_failures: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_success_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_failure_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_failure_status: Option<i64>,
    /// Why the most recent batch delivery failed, in the same error envelope `GET /jobs/{job_id}` returns for a failed job: a stable `code` from the shared error vocabulary, a fixed operator-facing `message`, and `retryable`. Null while no batch has failed, and cleared on the next success. Destination URLs, headers, credentials, response bodies, and internal exception text are never included. Nor is your drain URL written to Hedra's own logs, since it may carry authentication in its query string. `retryable` describes the condition, not what Hedra did: every failed batch is requeued until the drain auto-disables, so it answers whether fixing the destination and re-enabling is likely to help. Drains that last failed before this field became structured report `UNKNOWN`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<ErrorEnvelope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub updated_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by_key_id: Option<String>,
}

impl LogDrainConfig {
    pub fn builder() -> LogDrainConfigBuilder {
        <LogDrainConfigBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LogDrainConfigBuilder {
    id: Option<String>,
    name: Option<String>,
    url: Option<String>,
    format: Option<LogDrainFormat>,
    header_names: Option<Vec<String>>,
    enabled: Option<bool>,
    batch_size: Option<i64>,
    consecutive_failures: Option<i64>,
    last_success_at: Option<DateTime<FixedOffset>>,
    last_failure_at: Option<DateTime<FixedOffset>>,
    last_failure_status: Option<i64>,
    last_error: Option<ErrorEnvelope>,
    disabled_reason: Option<String>,
    created_at: Option<DateTime<FixedOffset>>,
    updated_at: Option<DateTime<FixedOffset>>,
    updated_by_key_id: Option<String>,
}

impl LogDrainConfigBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn format(mut self, value: LogDrainFormat) -> Self {
        self.format = Some(value);
        self
    }

    pub fn header_names(mut self, value: Vec<String>) -> Self {
        self.header_names = Some(value);
        self
    }

    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = Some(value);
        self
    }

    pub fn batch_size(mut self, value: i64) -> Self {
        self.batch_size = Some(value);
        self
    }

    pub fn consecutive_failures(mut self, value: i64) -> Self {
        self.consecutive_failures = Some(value);
        self
    }

    pub fn last_success_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.last_success_at = Some(value);
        self
    }

    pub fn last_failure_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.last_failure_at = Some(value);
        self
    }

    pub fn last_failure_status(mut self, value: i64) -> Self {
        self.last_failure_status = Some(value);
        self
    }

    pub fn last_error(mut self, value: ErrorEnvelope) -> Self {
        self.last_error = Some(value);
        self
    }

    pub fn disabled_reason(mut self, value: impl Into<String>) -> Self {
        self.disabled_reason = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn updated_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.updated_at = Some(value);
        self
    }

    pub fn updated_by_key_id(mut self, value: impl Into<String>) -> Self {
        self.updated_by_key_id = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`LogDrainConfig`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](LogDrainConfigBuilder::id)
    /// - [`name`](LogDrainConfigBuilder::name)
    /// - [`url`](LogDrainConfigBuilder::url)
    /// - [`format`](LogDrainConfigBuilder::format)
    /// - [`enabled`](LogDrainConfigBuilder::enabled)
    /// - [`batch_size`](LogDrainConfigBuilder::batch_size)
    /// - [`consecutive_failures`](LogDrainConfigBuilder::consecutive_failures)
    /// - [`created_at`](LogDrainConfigBuilder::created_at)
    /// - [`updated_at`](LogDrainConfigBuilder::updated_at)
    pub fn build(self) -> Result<LogDrainConfig, BuildError> {
        Ok(LogDrainConfig {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            url: self.url.ok_or_else(|| BuildError::missing_field("url"))?,
            format: self.format.ok_or_else(|| BuildError::missing_field("format"))?,
            header_names: self.header_names,
            enabled: self.enabled.ok_or_else(|| BuildError::missing_field("enabled"))?,
            batch_size: self.batch_size.ok_or_else(|| BuildError::missing_field("batch_size"))?,
            consecutive_failures: self.consecutive_failures.ok_or_else(|| BuildError::missing_field("consecutive_failures"))?,
            last_success_at: self.last_success_at,
            last_failure_at: self.last_failure_at,
            last_failure_status: self.last_failure_status,
            last_error: self.last_error,
            disabled_reason: self.disabled_reason,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            updated_at: self.updated_at.ok_or_else(|| BuildError::missing_field("updated_at"))?,
            updated_by_key_id: self.updated_by_key_id,
        })
    }
}
