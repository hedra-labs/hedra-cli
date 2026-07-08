pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ModelSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub modality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium: Option<bool>,
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
    modality: Option<String>,
    name: Option<String>,
    description: Option<String>,
    thumbnail_url: Option<String>,
    premium: Option<bool>,
}

impl ModelSummaryBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn modality(mut self, value: impl Into<String>) -> Self {
        self.modality = Some(value.into());
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

    pub fn thumbnail_url(mut self, value: impl Into<String>) -> Self {
        self.thumbnail_url = Some(value.into());
        self
    }

    pub fn premium(mut self, value: bool) -> Self {
        self.premium = Some(value);
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
            thumbnail_url: self.thumbnail_url,
            premium: self.premium,
        })
    }
}
