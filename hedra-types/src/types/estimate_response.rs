pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Price of a would-be submit, in US dollars.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct EstimateResponse {
    #[serde(default)]
    pub model: String,
    /// Price of a would-be submit.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub cost: f64,
    /// ISO-4217 currency code for `cost`.
    #[serde(default)]
    pub currency: String,
}

impl EstimateResponse {
    pub fn builder() -> EstimateResponseBuilder {
        <EstimateResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct EstimateResponseBuilder {
    model: Option<String>,
    cost: Option<f64>,
    currency: Option<String>,
}

impl EstimateResponseBuilder {
    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn cost(mut self, value: f64) -> Self {
        self.cost = Some(value);
        self
    }

    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`EstimateResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`model`](EstimateResponseBuilder::model)
    /// - [`cost`](EstimateResponseBuilder::cost)
    /// - [`currency`](EstimateResponseBuilder::currency)
    pub fn build(self) -> Result<EstimateResponse, BuildError> {
        Ok(EstimateResponse {
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            cost: self.cost.ok_or_else(|| BuildError::missing_field("cost"))?,
            currency: self.currency.ok_or_else(|| BuildError::missing_field("currency"))?,
        })
    }
}
