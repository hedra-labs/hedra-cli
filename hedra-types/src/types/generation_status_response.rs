pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationStatusResponse {
    /// ID of the generation.
    #[serde(default)]
    pub id: String,
    /// ID of the generated asset.
    #[serde(default)]
    pub asset_id: String,
    /// Type of generation.
    pub r#type: AssetType,
    /// Status of the generation.
    pub status: GenerationStatus,
    /// Current progress to completion. Between 0-1
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub progress: f64,
    /// Date the generation was submitted.
    #[serde(default)]
    pub created_at: String,
    /// Error message. Value is not present unless the status of the generation is 'error'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// URL of the generated asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// URL to download the asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// URL to stream the asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming_url: Option<String>,
    /// ID of the workspace this generation belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl GenerationStatusResponse {
    pub fn builder() -> GenerationStatusResponseBuilder {
        <GenerationStatusResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerationStatusResponseBuilder {
    id: Option<String>,
    asset_id: Option<String>,
    r#type: Option<AssetType>,
    status: Option<GenerationStatus>,
    progress: Option<f64>,
    created_at: Option<String>,
    error_message: Option<String>,
    url: Option<String>,
    download_url: Option<String>,
    streaming_url: Option<String>,
    workspace_id: Option<String>,
}

impl GenerationStatusResponseBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn asset_id(mut self, value: impl Into<String>) -> Self {
        self.asset_id = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: AssetType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn status(mut self, value: GenerationStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn progress(mut self, value: f64) -> Self {
        self.progress = Some(value);
        self
    }

    pub fn created_at(mut self, value: impl Into<String>) -> Self {
        self.created_at = Some(value.into());
        self
    }

    pub fn error_message(mut self, value: impl Into<String>) -> Self {
        self.error_message = Some(value.into());
        self
    }

    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn download_url(mut self, value: impl Into<String>) -> Self {
        self.download_url = Some(value.into());
        self
    }

    pub fn streaming_url(mut self, value: impl Into<String>) -> Self {
        self.streaming_url = Some(value.into());
        self
    }

    pub fn workspace_id(mut self, value: impl Into<String>) -> Self {
        self.workspace_id = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`GenerationStatusResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](GenerationStatusResponseBuilder::id)
    /// - [`asset_id`](GenerationStatusResponseBuilder::asset_id)
    /// - [`r#type`](GenerationStatusResponseBuilder::r#type)
    /// - [`status`](GenerationStatusResponseBuilder::status)
    /// - [`progress`](GenerationStatusResponseBuilder::progress)
    /// - [`created_at`](GenerationStatusResponseBuilder::created_at)
    pub fn build(self) -> Result<GenerationStatusResponse, BuildError> {
        Ok(GenerationStatusResponse {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            asset_id: self.asset_id.ok_or_else(|| BuildError::missing_field("asset_id"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            progress: self.progress.ok_or_else(|| BuildError::missing_field("progress"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            error_message: self.error_message,
            url: self.url,
            download_url: self.download_url,
            streaming_url: self.streaming_url,
            workspace_id: self.workspace_id,
        })
    }
}
