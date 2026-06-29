pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct AiModelPrice {
    /// Credit cost of the model.
    #[serde(default)]
    pub credit_cost: i64,
    /// Unit scaling for the cost.
    #[serde(default)]
    pub unit_scale: i64,
    /// Billing unit of the model (e.g. 'generation', 'second', 'character').
    #[serde(default)]
    pub billing_unit: String,
}

impl AiModelPrice {
    pub fn builder() -> AiModelPriceBuilder {
        <AiModelPriceBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct AiModelPriceBuilder {
    credit_cost: Option<i64>,
    unit_scale: Option<i64>,
    billing_unit: Option<String>,
}

impl AiModelPriceBuilder {
    pub fn credit_cost(mut self, value: i64) -> Self {
        self.credit_cost = Some(value);
        self
    }

    pub fn unit_scale(mut self, value: i64) -> Self {
        self.unit_scale = Some(value);
        self
    }

    pub fn billing_unit(mut self, value: impl Into<String>) -> Self {
        self.billing_unit = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`AiModelPrice`].
    /// This method will fail if any of the following fields are not set:
    /// - [`credit_cost`](AiModelPriceBuilder::credit_cost)
    /// - [`unit_scale`](AiModelPriceBuilder::unit_scale)
    /// - [`billing_unit`](AiModelPriceBuilder::billing_unit)
    pub fn build(self) -> Result<AiModelPrice, BuildError> {
        Ok(AiModelPrice {
            credit_cost: self.credit_cost.ok_or_else(|| BuildError::missing_field("credit_cost"))?,
            unit_scale: self.unit_scale.ok_or_else(|| BuildError::missing_field("unit_scale"))?,
            billing_unit: self.billing_unit.ok_or_else(|| BuildError::missing_field("billing_unit"))?,
        })
    }
}
