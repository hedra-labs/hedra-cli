pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LogDrainCreate {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<LogDrainFormat>,
    /// Signs every NDJSON post. Required when `format` is `ndjson` (the default); optional for `otlp` drains, whose receivers authenticate with `headers` instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<i64>,
}

impl LogDrainCreate {
    pub fn builder() -> LogDrainCreateBuilder {
        <LogDrainCreateBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LogDrainCreateBuilder {
    name: Option<String>,
    url: Option<String>,
    format: Option<LogDrainFormat>,
    secret: Option<String>,
    headers: Option<HashMap<String, Option<String>>>,
    enabled: Option<bool>,
    batch_size: Option<i64>,
}

impl LogDrainCreateBuilder {
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

    pub fn secret(mut self, value: impl Into<String>) -> Self {
        self.secret = Some(value.into());
        self
    }

    pub fn headers(mut self, value: HashMap<String, Option<String>>) -> Self {
        self.headers = Some(value);
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

    /// Consumes the builder and constructs a [`LogDrainCreate`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](LogDrainCreateBuilder::name)
    /// - [`url`](LogDrainCreateBuilder::url)
    pub fn build(self) -> Result<LogDrainCreate, BuildError> {
        Ok(LogDrainCreate {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            url: self.url.ok_or_else(|| BuildError::missing_field("url"))?,
            format: self.format,
            secret: self.secret,
            headers: self.headers,
            enabled: self.enabled,
            batch_size: self.batch_size,
        })
    }
}

