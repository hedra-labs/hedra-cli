pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for list_assets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ListAssetsQueryRequest {
    pub r#type: AssetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<String>,
}

impl ListAssetsQueryRequest {
    pub fn builder() -> ListAssetsQueryRequestBuilder {
        <ListAssetsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ListAssetsQueryRequestBuilder {
    r#type: Option<AssetType>,
    ids: Option<String>,
}

impl ListAssetsQueryRequestBuilder {
    pub fn r#type(mut self, value: AssetType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn ids(mut self, value: impl Into<String>) -> Self {
        self.ids = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ListAssetsQueryRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`r#type`](ListAssetsQueryRequestBuilder::r#type)
    pub fn build(self) -> Result<ListAssetsQueryRequest, BuildError> {
        Ok(ListAssetsQueryRequest {
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            ids: self.ids,
        })
    }
}

