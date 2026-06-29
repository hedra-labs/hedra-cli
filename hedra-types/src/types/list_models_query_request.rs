pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for list_models
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ListModelsQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
}

impl ListModelsQueryRequest {
    pub fn builder() -> ListModelsQueryRequestBuilder {
        <ListModelsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ListModelsQueryRequestBuilder {
    types: Option<Vec<String>>,
}

impl ListModelsQueryRequestBuilder {
    pub fn types(mut self, value: Vec<String>) -> Self {
        self.types = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ListModelsQueryRequest`].
    pub fn build(self) -> Result<ListModelsQueryRequest, BuildError> {
        Ok(ListModelsQueryRequest {
            types: self.types,
        })
    }
}

