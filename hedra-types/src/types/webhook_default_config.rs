pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct WebhookDefaultConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by_key_id: Option<String>,
}

impl WebhookDefaultConfig {
    pub fn builder() -> WebhookDefaultConfigBuilder {
        <WebhookDefaultConfigBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookDefaultConfigBuilder {
    url: Option<String>,
    enabled: Option<bool>,
    updated_at: Option<DateTime<FixedOffset>>,
    updated_by_key_id: Option<String>,
}

impl WebhookDefaultConfigBuilder {
    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = Some(value);
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

    /// Consumes the builder and constructs a [`WebhookDefaultConfig`].
    pub fn build(self) -> Result<WebhookDefaultConfig, BuildError> {
        Ok(WebhookDefaultConfig {
            url: self.url,
            enabled: self.enabled,
            updated_at: self.updated_at,
            updated_by_key_id: self.updated_by_key_id,
        })
    }
}
