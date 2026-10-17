pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A negotiated discount the key's workspace is charged under now.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Discount {
    /// The model id the discount applies to.
    #[serde(default)]
    pub model: String,
    /// Percentage off the model's list price, e.g. 20 or 12.5.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub percent_off: f64,
    /// When the discount took effect.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub starts_at: DateTime<FixedOffset>,
    /// When the discount ends; null when open-ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<DateTime<FixedOffset>>,
}

impl Discount {
    pub fn builder() -> DiscountBuilder {
        <DiscountBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct DiscountBuilder {
    model: Option<String>,
    percent_off: Option<f64>,
    starts_at: Option<DateTime<FixedOffset>>,
    ends_at: Option<DateTime<FixedOffset>>,
}

impl DiscountBuilder {
    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn percent_off(mut self, value: f64) -> Self {
        self.percent_off = Some(value);
        self
    }

    pub fn starts_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.starts_at = Some(value);
        self
    }

    pub fn ends_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.ends_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`Discount`].
    /// This method will fail if any of the following fields are not set:
    /// - [`model`](DiscountBuilder::model)
    /// - [`percent_off`](DiscountBuilder::percent_off)
    /// - [`starts_at`](DiscountBuilder::starts_at)
    pub fn build(self) -> Result<Discount, BuildError> {
        Ok(Discount {
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            percent_off: self.percent_off.ok_or_else(|| BuildError::missing_field("percent_off"))?,
            starts_at: self.starts_at.ok_or_else(|| BuildError::missing_field("starts_at"))?,
            ends_at: self.ends_at,
        })
    }
}
