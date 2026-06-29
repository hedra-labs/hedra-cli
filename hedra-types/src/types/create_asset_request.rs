pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CreateAssetRequest {
    /// Name of the asset. Default to user-provided file name.
    #[serde(default)]
    pub name: String,
    /// The type of the asset.
    pub r#type: AssetType,
}

impl CreateAssetRequest {
    pub fn builder() -> CreateAssetRequestBuilder {
        <CreateAssetRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct CreateAssetRequestBuilder {
    name: Option<String>,
    r#type: Option<AssetType>,
}

impl CreateAssetRequestBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: AssetType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`CreateAssetRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](CreateAssetRequestBuilder::name)
    /// - [`r#type`](CreateAssetRequestBuilder::r#type)
    pub fn build(self) -> Result<CreateAssetRequest, BuildError> {
        Ok(CreateAssetRequest {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
        })
    }
}

