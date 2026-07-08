pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for list
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ModelsListQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

impl ModelsListQueryRequest {
    pub fn builder() -> ModelsListQueryRequestBuilder {
        <ModelsListQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ModelsListQueryRequestBuilder {
    r#type: Option<String>,
}

impl ModelsListQueryRequestBuilder {
    pub fn r#type(mut self, value: impl Into<String>) -> Self {
        self.r#type = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ModelsListQueryRequest`].
    pub fn build(self) -> Result<ModelsListQueryRequest, BuildError> {
        Ok(ModelsListQueryRequest {
            r#type: self.r#type,
        })
    }
}

