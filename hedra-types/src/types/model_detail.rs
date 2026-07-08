pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ModelDetail {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub modality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<Vec<ModelRoute>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<ModelVariant>>,
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
    kind: Option<String>,
    modality: Option<String>,
    name: Option<String>,
    description: Option<String>,
    input_schema: Option<HashMap<String, serde_json::Value>>,
    output_schema: Option<HashMap<String, serde_json::Value>>,
    routing: Option<Vec<ModelRoute>>,
    variants: Option<Vec<ModelVariant>>,
}

impl ModelDetailBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn kind(mut self, value: impl Into<String>) -> Self {
        self.kind = Some(value.into());
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

    pub fn input_schema(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.input_schema = Some(value);
        self
    }

    pub fn output_schema(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.output_schema = Some(value);
        self
    }

    pub fn routing(mut self, value: Vec<ModelRoute>) -> Self {
        self.routing = Some(value);
        self
    }

    pub fn variants(mut self, value: Vec<ModelVariant>) -> Self {
        self.variants = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ModelDetail`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](ModelDetailBuilder::id)
    /// - [`kind`](ModelDetailBuilder::kind)
    /// - [`modality`](ModelDetailBuilder::modality)
    pub fn build(self) -> Result<ModelDetail, BuildError> {
        Ok(ModelDetail {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            kind: self.kind.ok_or_else(|| BuildError::missing_field("kind"))?,
            modality: self.modality.ok_or_else(|| BuildError::missing_field("modality"))?,
            name: self.name,
            description: self.description,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
            routing: self.routing,
            variants: self.variants,
        })
    }
}
