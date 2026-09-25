pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Price per one million tokens, in USD.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LlmPricing {
    #[serde(rename = "usd_per_1m_input_tokens")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub usd_per1m_input_tokens: f64,
    #[serde(rename = "usd_per_1m_output_tokens")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub usd_per1m_output_tokens: f64,
}

impl LlmPricing {
    pub fn builder() -> LlmPricingBuilder {
        <LlmPricingBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LlmPricingBuilder {
    usd_per1m_input_tokens: Option<f64>,
    usd_per1m_output_tokens: Option<f64>,
}

impl LlmPricingBuilder {
    pub fn usd_per1m_input_tokens(mut self, value: f64) -> Self {
        self.usd_per1m_input_tokens = Some(value);
        self
    }

    pub fn usd_per1m_output_tokens(mut self, value: f64) -> Self {
        self.usd_per1m_output_tokens = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`LlmPricing`].
    /// This method will fail if any of the following fields are not set:
    /// - [`usd_per1m_input_tokens`](LlmPricingBuilder::usd_per1m_input_tokens)
    /// - [`usd_per1m_output_tokens`](LlmPricingBuilder::usd_per1m_output_tokens)
    pub fn build(self) -> Result<LlmPricing, BuildError> {
        Ok(LlmPricing {
            usd_per1m_input_tokens: self.usd_per1m_input_tokens.ok_or_else(|| BuildError::missing_field("usd_per1m_input_tokens"))?,
            usd_per1m_output_tokens: self.usd_per1m_output_tokens.ok_or_else(|| BuildError::missing_field("usd_per1m_output_tokens"))?,
        })
    }
}
