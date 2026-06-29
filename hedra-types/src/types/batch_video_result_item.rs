pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Individual result item in a batch video generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchVideoResultItem {
    /// The id of the generation created. None if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The id of the video asset resulting from the generation. None if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// Date the generation was submitted.
    #[serde(default)]
    pub created_at: String,
    /// Status of the generation
    pub status: GenerationStatus,
    /// Current progress to completion. Between 0-1
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub progress: f64,
    /// Error message if this item failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl BatchVideoResultItem {
    pub fn builder() -> BatchVideoResultItemBuilder {
        <BatchVideoResultItemBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct BatchVideoResultItemBuilder {
    id: Option<String>,
    asset_id: Option<String>,
    created_at: Option<String>,
    status: Option<GenerationStatus>,
    progress: Option<f64>,
    error: Option<String>,
}

impl BatchVideoResultItemBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn asset_id(mut self, value: impl Into<String>) -> Self {
        self.asset_id = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: impl Into<String>) -> Self {
        self.created_at = Some(value.into());
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

    pub fn error(mut self, value: impl Into<String>) -> Self {
        self.error = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`BatchVideoResultItem`].
    /// This method will fail if any of the following fields are not set:
    /// - [`created_at`](BatchVideoResultItemBuilder::created_at)
    /// - [`status`](BatchVideoResultItemBuilder::status)
    /// - [`progress`](BatchVideoResultItemBuilder::progress)
    pub fn build(self) -> Result<BatchVideoResultItem, BuildError> {
        Ok(BatchVideoResultItem {
            id: self.id,
            asset_id: self.asset_id,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            progress: self.progress.ok_or_else(|| BuildError::missing_field("progress"))?,
            error: self.error,
        })
    }
}
