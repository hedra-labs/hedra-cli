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
    /// Short USD pricing summary for this model. Exact cost depends on input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_description: Option<String>,
    /// URL of the provider's logo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// A chat model's context window in tokens; null for other models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<i64>,
    /// The largest `max_tokens` a chat model accepts; null for other models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,
    /// JSON Schema for this model's `input` object on submit — the same schema `GET /v3/models/{model}/openapi.json` embeds. For a chat model, the JSON Schema of the `POST /v3/chat/completions` request body as this model accepts it; a chat model has no per-model OpenAPI document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<HashMap<String, serde_json::Value>>,
    /// JSON Schema of one item of a completed job's `outputs[]`. Empty for a chat model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<HashMap<String, serde_json::Value>>,
    /// The facts about a chat model that its `input_schema` cannot state: token prices, the default output bound and the image limit; null for other models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<ChatCapabilities>,
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
    price_description: Option<String>,
    logo_url: Option<String>,
    context_length: Option<i64>,
    max_output_tokens: Option<i64>,
    input_schema: Option<HashMap<String, serde_json::Value>>,
    output_schema: Option<HashMap<String, serde_json::Value>>,
    chat: Option<ChatCapabilities>,
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

    pub fn price_description(mut self, value: impl Into<String>) -> Self {
        self.price_description = Some(value.into());
        self
    }

    pub fn logo_url(mut self, value: impl Into<String>) -> Self {
        self.logo_url = Some(value.into());
        self
    }

    pub fn context_length(mut self, value: i64) -> Self {
        self.context_length = Some(value);
        self
    }

    pub fn max_output_tokens(mut self, value: i64) -> Self {
        self.max_output_tokens = Some(value);
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

    pub fn chat(mut self, value: ChatCapabilities) -> Self {
        self.chat = Some(value);
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
            price_description: self.price_description,
            logo_url: self.logo_url,
            context_length: self.context_length,
            max_output_tokens: self.max_output_tokens,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
            chat: self.chat,
        })
    }
}
