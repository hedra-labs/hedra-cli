pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelDetail {
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
    /// URL of the provider's logo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// JSON Schema for this model's `input` object on submit — the same schema `GET /v3/models/{model}/openapi.json` embeds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<HashMap<String, serde_json::Value>>,
    /// JSON Schema of one item of a completed job's `outputs[]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<HashMap<String, serde_json::Value>>,
}

impl ModelDetail {
    pub fn builder() -> ModelDetailBuilder {
        <ModelDetailBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ModelDetailBuilder {
    id: Option<String>,
    modality: Option<Modality>,
    name: Option<String>,
    description: Option<String>,
    logo_url: Option<String>,
    input_schema: Option<HashMap<String, serde_json::Value>>,
    output_schema: Option<HashMap<String, serde_json::Value>>,
}

impl ModelDetailBuilder {
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

    pub fn logo_url(mut self, value: impl Into<String>) -> Self {
        self.logo_url = Some(value.into());
        self
    }

    pub fn input_schema(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.input_schema = Some(value);
        self
    }

    pub fn output_schema(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.output_schema = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ModelDetail`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](ModelDetailBuilder::id)
    /// - [`modality`](ModelDetailBuilder::modality)
    pub fn build(self) -> Result<ModelDetail, BuildError> {
        Ok(ModelDetail {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            modality: self.modality.ok_or_else(|| BuildError::missing_field("modality"))?,
            name: self.name,
            description: self.description,
            logo_url: self.logo_url,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
        })
    }
}
