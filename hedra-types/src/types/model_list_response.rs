pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ModelListResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<ModelSummary>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl ModelListResponse {
    pub fn builder() -> ModelListResponseBuilder {
        <ModelListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ModelListResponseBuilder {
    data: Option<Vec<ModelSummary>>,
    next_cursor: Option<String>,
}

impl ModelListResponseBuilder {
    pub fn data(mut self, value: Vec<ModelSummary>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ModelListResponse`].
    pub fn build(self) -> Result<ModelListResponse, BuildError> {
        Ok(ModelListResponse {
            data: self.data,
            next_cursor: self.next_cursor,
        })
    }
}
