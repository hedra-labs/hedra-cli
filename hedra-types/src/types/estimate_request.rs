pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct EstimateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<HashMap<String, serde_json::Value>>,
}

impl EstimateRequest {
    pub fn builder() -> EstimateRequestBuilder {
        <EstimateRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct EstimateRequestBuilder {
    input: Option<HashMap<String, serde_json::Value>>,
}

impl EstimateRequestBuilder {
    pub fn input(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.input = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`EstimateRequest`].
    pub fn build(self) -> Result<EstimateRequest, BuildError> {
        Ok(EstimateRequest {
            input: self.input,
        })
    }
}

