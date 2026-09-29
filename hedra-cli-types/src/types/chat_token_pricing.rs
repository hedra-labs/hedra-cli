pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ChatTokenPricing {
    /// USD per million prompt tokens, cached tokens included.
    #[serde(rename = "usd_per_1m_input_tokens")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub usd_per1m_input_tokens: f64,
    /// USD per million completion tokens, reasoning tokens included.
    #[serde(rename = "usd_per_1m_output_tokens")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub usd_per1m_output_tokens: f64,
}

impl ChatTokenPricing {
    pub fn builder() -> ChatTokenPricingBuilder {
        <ChatTokenPricingBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ChatTokenPricingBuilder {
    usd_per1m_input_tokens: Option<f64>,
    usd_per1m_output_tokens: Option<f64>,
}

impl ChatTokenPricingBuilder {
    pub fn usd_per1m_input_tokens(mut self, value: f64) -> Self {
        self.usd_per1m_input_tokens = Some(value);
        self
    }

    pub fn usd_per1m_output_tokens(mut self, value: f64) -> Self {
        self.usd_per1m_output_tokens = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ChatTokenPricing`].
    /// This method will fail if any of the following fields are not set:
    /// - [`usd_per1m_input_tokens`](ChatTokenPricingBuilder::usd_per1m_input_tokens)
    /// - [`usd_per1m_output_tokens`](ChatTokenPricingBuilder::usd_per1m_output_tokens)
    pub fn build(self) -> Result<ChatTokenPricing, BuildError> {
        Ok(ChatTokenPricing {
            usd_per1m_input_tokens: self.usd_per1m_input_tokens.ok_or_else(|| BuildError::missing_field("usd_per1m_input_tokens"))?,
            usd_per1m_output_tokens: self.usd_per1m_output_tokens.ok_or_else(|| BuildError::missing_field("usd_per1m_output_tokens"))?,
        })
    }
}
