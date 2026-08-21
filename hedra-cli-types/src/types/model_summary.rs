pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModelSummary {
    /// The model's public id — the value POST /v3/models/{model} takes.
    #[serde(default)]
    pub id: String,
    pub modality: Modality,
    /// Human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// One-line summary of what the model does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Short USD pricing summary for this model. Exact cost depends on input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_description: Option<String>,
    /// URL of the provider's logo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

impl ModelSummary {
    pub fn builder() -> ModelSummaryBuilder {
        <ModelSummaryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ModelSummaryBuilder {
    id: Option<String>,
    modality: Option<Modality>,
    name: Option<String>,
    description: Option<String>,
    price_description: Option<String>,
    logo_url: Option<String>,
}

impl ModelSummaryBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn modality(mut self, value: Modality) -> Self {
        self.modality = Some(value);
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn price_description(mut self, value: impl Into<String>) -> Self {
        self.price_description = Some(value.into());
        self
    }

    pub fn logo_url(mut self, value: impl Into<String>) -> Self {
        self.logo_url = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ModelSummary`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](ModelSummaryBuilder::id)
    /// - [`modality`](ModelSummaryBuilder::modality)
    pub fn build(self) -> Result<ModelSummary, BuildError> {
        Ok(ModelSummary {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            modality: self.modality.ok_or_else(|| BuildError::missing_field("modality"))?,
            name: self.name,
            description: self.description,
            price_description: self.price_description,
            logo_url: self.logo_url,
        })
    }
}
