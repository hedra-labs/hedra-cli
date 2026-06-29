pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    /// The id of the asset.
    #[serde(default)]
    pub id: String,
    /// The type of the asset.
    pub r#type: AssetType,
    /// Name of the asset. Default to user-provided file name.
    #[serde(default)]
    pub name: String,
    /// URL of the thumbnail image.
    #[serde(default)]
    pub thumbnail_url: String,
    /// Optional description of the asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the asset is favorited by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
    /// Whether this asset was recently used by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent: Option<bool>,
    /// Date the asset was created.
    #[serde(default)]
    pub created_at: String,
    /// Date the asset was favorited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorited_at: Option<String>,
    /// The asset itself.
    pub asset: Box<AssetAsset>,
}

impl Asset {
    pub fn builder() -> AssetBuilder {
        <AssetBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct AssetBuilder {
    id: Option<String>,
    r#type: Option<AssetType>,
    name: Option<String>,
    thumbnail_url: Option<String>,
    description: Option<String>,
    is_favorite: Option<bool>,
    recent: Option<bool>,
    created_at: Option<String>,
    favorited_at: Option<String>,
    asset: Option<Box<AssetAsset>>,
}

impl AssetBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: AssetType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn thumbnail_url(mut self, value: impl Into<String>) -> Self {
        self.thumbnail_url = Some(value.into());
        self
    }

    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn is_favorite(mut self, value: bool) -> Self {
        self.is_favorite = Some(value);
        self
    }

    pub fn recent(mut self, value: bool) -> Self {
        self.recent = Some(value);
        self
    }

    pub fn created_at(mut self, value: impl Into<String>) -> Self {
        self.created_at = Some(value.into());
        self
    }

    pub fn favorited_at(mut self, value: impl Into<String>) -> Self {
        self.favorited_at = Some(value.into());
        self
    }

    pub fn asset(mut self, value: Box<AssetAsset>) -> Self {
        self.asset = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`Asset`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](AssetBuilder::id)
    /// - [`r#type`](AssetBuilder::r#type)
    /// - [`name`](AssetBuilder::name)
    /// - [`thumbnail_url`](AssetBuilder::thumbnail_url)
    /// - [`created_at`](AssetBuilder::created_at)
    /// - [`asset`](AssetBuilder::asset)
    pub fn build(self) -> Result<Asset, BuildError> {
        Ok(Asset {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            thumbnail_url: self.thumbnail_url.ok_or_else(|| BuildError::missing_field("thumbnail_url"))?,
            description: self.description,
            is_favorite: self.is_favorite,
            recent: self.recent,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            favorited_at: self.favorited_at,
            asset: self.asset.ok_or_else(|| BuildError::missing_field("asset"))?,
        })
    }
}
