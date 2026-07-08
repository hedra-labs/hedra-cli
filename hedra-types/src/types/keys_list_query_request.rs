pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for list
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct KeysListQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl KeysListQueryRequest {
    pub fn builder() -> KeysListQueryRequestBuilder {
        <KeysListQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KeysListQueryRequestBuilder {
    workspace_id: Option<String>,
}

impl KeysListQueryRequestBuilder {
    pub fn workspace_id(mut self, value: impl Into<String>) -> Self {
        self.workspace_id = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`KeysListQueryRequest`].
    pub fn build(self) -> Result<KeysListQueryRequest, BuildError> {
        Ok(KeysListQueryRequest {
            workspace_id: self.workspace_id,
        })
    }
}

