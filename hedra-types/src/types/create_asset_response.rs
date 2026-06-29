pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CreateAssetResponse {
    /// Name of the asset. Default to user-provided file name.
    #[serde(default)]
    pub name: String,
    /// The type of the asset.
    pub r#type: AssetType,
    /// The id of the newly created asset. Should be used for upload.
    #[serde(default)]
    pub id: String,
    /// Presigned S3 URL for uploading the asset file using HTTP PUT. Only present when the asset type is VIDEO.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_url: Option<String>,
}

impl CreateAssetResponse {
    pub fn builder() -> CreateAssetResponseBuilder {
        <CreateAssetResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct CreateAssetResponseBuilder {
    name: Option<String>,
    r#type: Option<AssetType>,
    id: Option<String>,
    upload_url: Option<String>,
}

impl CreateAssetResponseBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: AssetType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn upload_url(mut self, value: impl Into<String>) -> Self {
        self.upload_url = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`CreateAssetResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](CreateAssetResponseBuilder::name)
    /// - [`r#type`](CreateAssetResponseBuilder::r#type)
    /// - [`id`](CreateAssetResponseBuilder::id)
    pub fn build(self) -> Result<CreateAssetResponse, BuildError> {
        Ok(CreateAssetResponse {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            upload_url: self.upload_url,
        })
    }
}
