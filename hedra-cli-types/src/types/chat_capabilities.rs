pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ChatCapabilities {
    /// The output bound of a request that sets neither `max_tokens` nor `max_completion_tokens`.
    #[serde(default)]
    pub default_max_tokens: i64,
    #[serde(default)]
    pub pricing: ChatTokenPricing,
    /// `text`, and `image` when the model accepts image parts.
    #[serde(default)]
    pub input_modalities: Vec<String>,
    /// The most image parts one request may contain; null when the model accepts no images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_images_per_request: Option<i64>,
}

impl ChatCapabilities {
    pub fn builder() -> ChatCapabilitiesBuilder {
        <ChatCapabilitiesBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ChatCapabilitiesBuilder {
    default_max_tokens: Option<i64>,
    pricing: Option<ChatTokenPricing>,
    input_modalities: Option<Vec<String>>,
    max_images_per_request: Option<i64>,
}

impl ChatCapabilitiesBuilder {
    pub fn default_max_tokens(mut self, value: i64) -> Self {
        self.default_max_tokens = Some(value);
        self
    }

    pub fn pricing(mut self, value: ChatTokenPricing) -> Self {
        self.pricing = Some(value);
        self
    }

    pub fn input_modalities(mut self, value: Vec<String>) -> Self {
        self.input_modalities = Some(value);
        self
    }

    pub fn max_images_per_request(mut self, value: i64) -> Self {
        self.max_images_per_request = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ChatCapabilities`].
    /// This method will fail if any of the following fields are not set:
    /// - [`default_max_tokens`](ChatCapabilitiesBuilder::default_max_tokens)
    /// - [`pricing`](ChatCapabilitiesBuilder::pricing)
    /// - [`input_modalities`](ChatCapabilitiesBuilder::input_modalities)
    pub fn build(self) -> Result<ChatCapabilities, BuildError> {
        Ok(ChatCapabilities {
            default_max_tokens: self.default_max_tokens.ok_or_else(|| BuildError::missing_field("default_max_tokens"))?,
            pricing: self.pricing.ok_or_else(|| BuildError::missing_field("pricing"))?,
            input_modalities: self.input_modalities.ok_or_else(|| BuildError::missing_field("input_modalities"))?,
            max_images_per_request: self.max_images_per_request,
        })
    }
}
