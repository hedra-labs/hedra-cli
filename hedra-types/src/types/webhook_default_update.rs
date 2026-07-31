pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct WebhookDefaultUpdate {
    #[serde(default)]
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl WebhookDefaultUpdate {
    pub fn builder() -> WebhookDefaultUpdateBuilder {
        <WebhookDefaultUpdateBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookDefaultUpdateBuilder {
    url: Option<String>,
    enabled: Option<bool>,
}

impl WebhookDefaultUpdateBuilder {
    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`WebhookDefaultUpdate`].
    /// This method will fail if any of the following fields are not set:
    /// - [`url`](WebhookDefaultUpdateBuilder::url)
    pub fn build(self) -> Result<WebhookDefaultUpdate, BuildError> {
        Ok(WebhookDefaultUpdate {
            url: self.url.ok_or_else(|| BuildError::missing_field("url"))?,
            enabled: self.enabled,
        })
    }
}

