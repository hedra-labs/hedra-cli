pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Generation {
    /// ID of the generation and associated asset.
    #[serde(default)]
    pub id: String,
    /// Type of generation.
    pub r#type: AssetType,
    /// Inputs for the generation
    pub input: GenerationInput,
    /// Status of the generation
    pub status: GenerationStatus,
    /// Current progress to completion. Between 0-1
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub progress: f64,
    /// Estimated time remaining in seconds until generation completes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_sec: Option<i64>,
    /// Date the generation was submitted.
    #[serde(default)]
    pub created_at: String,
    /// Credits consumed (debits) for this generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_cost: Option<i64>,
    /// Unique identifier linking all generations in a batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_generation_id: Option<String>,
    /// The generated asset. Value is not present unless the status of the generation is 'complete'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<Asset>,
    /// Generation error if any. Value is not present unless the status of the generation is 'error' and error_message field is not present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<GenerationError>,
    /// Error message. Value is not present unless the status of the generation is 'error' and error field is not present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl Generation {
    pub fn builder() -> GenerationBuilder {
        <GenerationBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerationBuilder {
    id: Option<String>,
    r#type: Option<AssetType>,
    input: Option<GenerationInput>,
    status: Option<GenerationStatus>,
    progress: Option<f64>,
    eta_sec: Option<i64>,
    created_at: Option<String>,
    credit_cost: Option<i64>,
    batch_generation_id: Option<String>,
    asset: Option<Asset>,
    error: Option<GenerationError>,
    error_message: Option<String>,
}

impl GenerationBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: AssetType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn input(mut self, value: GenerationInput) -> Self {
        self.input = Some(value);
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

    pub fn eta_sec(mut self, value: i64) -> Self {
        self.eta_sec = Some(value);
        self
    }

    pub fn created_at(mut self, value: impl Into<String>) -> Self {
        self.created_at = Some(value.into());
        self
    }

    pub fn credit_cost(mut self, value: i64) -> Self {
        self.credit_cost = Some(value);
        self
    }

    pub fn batch_generation_id(mut self, value: impl Into<String>) -> Self {
        self.batch_generation_id = Some(value.into());
        self
    }

    pub fn asset(mut self, value: Asset) -> Self {
        self.asset = Some(value);
        self
    }

    pub fn error(mut self, value: GenerationError) -> Self {
        self.error = Some(value);
        self
    }

    pub fn error_message(mut self, value: impl Into<String>) -> Self {
        self.error_message = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`Generation`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](GenerationBuilder::id)
    /// - [`r#type`](GenerationBuilder::r#type)
    /// - [`input`](GenerationBuilder::input)
    /// - [`status`](GenerationBuilder::status)
    /// - [`progress`](GenerationBuilder::progress)
    /// - [`created_at`](GenerationBuilder::created_at)
    pub fn build(self) -> Result<Generation, BuildError> {
        Ok(Generation {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            progress: self.progress.ok_or_else(|| BuildError::missing_field("progress"))?,
            eta_sec: self.eta_sec,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            credit_cost: self.credit_cost,
            batch_generation_id: self.batch_generation_id,
            asset: self.asset,
            error: self.error,
            error_message: self.error_message,
        })
    }
}
