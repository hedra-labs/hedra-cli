pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LogDrainUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<LogDrainFormat>,
    /// Rotates the signing secret. No conditional applies here: the drain may already hold one. Switching `format` to `ndjson` on a drain with no stored secret requires supplying one in the same request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<i64>,
}

impl LogDrainUpdate {
    pub fn builder() -> LogDrainUpdateBuilder {
        <LogDrainUpdateBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LogDrainUpdateBuilder {
    name: Option<String>,
    url: Option<String>,
    format: Option<LogDrainFormat>,
    secret: Option<String>,
    headers: Option<HashMap<String, Option<String>>>,
    enabled: Option<bool>,
    batch_size: Option<i64>,
}

impl LogDrainUpdateBuilder {
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

    /// Consumes the builder and constructs a [`LogDrainUpdate`].
    pub fn build(self) -> Result<LogDrainUpdate, BuildError> {
        Ok(LogDrainUpdate {
            name: self.name,
            url: self.url,
            format: self.format,
            secret: self.secret,
            headers: self.headers,
            enabled: self.enabled,
            batch_size: self.batch_size,
        })
    }
}

