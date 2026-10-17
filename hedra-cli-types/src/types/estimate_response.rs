pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The price, in US dollars, of submitting the request's `input` to the model.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct EstimateResponse {
    /// The resolved model id this estimate prices.
    #[serde(default)]
    pub model: String,
    /// The price of submitting the request's `input` to the model, in `currency`, after any discount negotiated for the key's workspace.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub cost: f64,
    /// The model's list price for the same request, in `currency`, before any negotiated discount; equal to `cost` when none applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_cost: Option<f64>,
    /// The negotiated percentage off the model's list price that `cost` reflects, e.g. 20 or 12.5; null when none applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<f64>,
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
    list_cost: Option<f64>,
    discount_percent: Option<f64>,
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

    pub fn list_cost(mut self, value: f64) -> Self {
        self.list_cost = Some(value);
        self
    }

    pub fn discount_percent(mut self, value: f64) -> Self {
        self.discount_percent = Some(value);
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
            list_cost: self.list_cost,
            discount_percent: self.discount_percent,
            currency: self.currency.ok_or_else(|| BuildError::missing_field("currency"))?,
        })
    }
}
