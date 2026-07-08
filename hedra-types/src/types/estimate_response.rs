pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct EstimateResponse {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub credits: i64,
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
    credits: Option<i64>,
}

impl EstimateResponseBuilder {
    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn credits(mut self, value: i64) -> Self {
        self.credits = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`EstimateResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`model`](EstimateResponseBuilder::model)
    /// - [`credits`](EstimateResponseBuilder::credits)
    pub fn build(self) -> Result<EstimateResponse, BuildError> {
        Ok(EstimateResponse {
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            credits: self.credits.ok_or_else(|| BuildError::missing_field("credits"))?,
        })
    }
}
