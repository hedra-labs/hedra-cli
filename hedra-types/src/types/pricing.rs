pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Extensible pricing information for AI models.
/// 
/// Price is calculated as:
/// credits = ceil((base_cost + credits_per_second * duration_seconds) * product(modifiers))
/// 
/// For video models (per-second billing):
/// - base_cost is typically 0
/// - credits_per_second is the per-second rate
/// - duration_seconds is the video length
/// - modifiers adjust for resolution, audio generation, etc.
/// 
/// For image models (per-generation billing):
/// - base_cost is the flat cost per generation (e.g., 5 credits)
/// - credits_per_second is 0
/// - duration_seconds is ignored (or 0)
/// - modifiers can still adjust for resolution, style, etc.
/// 
/// Modifiers are only included when they affect pricing (i.e., non-1.0 values).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Pricing {
    /// Flat base cost in credits. Present for per-generation models (images). Duration-based models leave this unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_cost: Option<i64>,
    /// Credit cost per second. Only present for duration-based models. Image/per-generation models leave this unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits_per_second: Option<i64>,
    /// Pricing modifiers by dimension. Each dimension (e.g. 'resolution', 'generate_audio') maps option values to their price multipliers. Only non-1.0 modifiers are included (1.0 is the implicit default). Example: {'resolution': {'540p': 0.5, '1080p': 3.0}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<HashMap<String, HashMap<String, f64>>>,
}

impl Pricing {
    pub fn builder() -> PricingBuilder {
        <PricingBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct PricingBuilder {
    base_cost: Option<i64>,
    credits_per_second: Option<i64>,
    modifiers: Option<HashMap<String, HashMap<String, f64>>>,
}

impl PricingBuilder {
    pub fn base_cost(mut self, value: i64) -> Self {
        self.base_cost = Some(value);
        self
    }

    pub fn credits_per_second(mut self, value: i64) -> Self {
        self.credits_per_second = Some(value);
        self
    }

    pub fn modifiers(mut self, value: HashMap<String, HashMap<String, f64>>) -> Self {
        self.modifiers = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`Pricing`].
    pub fn build(self) -> Result<Pricing, BuildError> {
        Ok(Pricing {
            base_cost: self.base_cost,
            credits_per_second: self.credits_per_second,
            modifiers: self.modifiers,
        })
    }
}
