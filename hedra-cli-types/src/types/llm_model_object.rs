pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// OpenAI model-object base + additive extensions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmModelObject {
    #[serde(default)]
    pub id: String,
    pub object: LlmModelObjectObject,
    /// Unix seconds the entry was authored — the OpenAI spec's shape.
    #[serde(default)]
    pub created: i64,
    /// The model creator organization, never the serving operator.
    #[serde(default)]
    pub owned_by: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub context_length: i64,
    #[serde(default)]
    pub max_output_tokens: i64,
    #[serde(default)]
    pub capabilities: LlmCapabilitiesObject,
    #[serde(default)]
    pub pricing: LlmPricing,
}

impl LlmModelObject {
    pub fn builder() -> LlmModelObjectBuilder {
        <LlmModelObjectBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LlmModelObjectBuilder {
    id: Option<String>,
    object: Option<LlmModelObjectObject>,
    created: Option<i64>,
    owned_by: Option<String>,
    name: Option<String>,
    description: Option<String>,
    context_length: Option<i64>,
    max_output_tokens: Option<i64>,
    capabilities: Option<LlmCapabilitiesObject>,
    pricing: Option<LlmPricing>,
}

impl LlmModelObjectBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn object(mut self, value: LlmModelObjectObject) -> Self {
        self.object = Some(value);
        self
    }

    pub fn created(mut self, value: i64) -> Self {
        self.created = Some(value);
        self
    }

    pub fn owned_by(mut self, value: impl Into<String>) -> Self {
        self.owned_by = Some(value.into());
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

    pub fn context_length(mut self, value: i64) -> Self {
        self.context_length = Some(value);
        self
    }

    pub fn max_output_tokens(mut self, value: i64) -> Self {
        self.max_output_tokens = Some(value);
        self
    }

    pub fn capabilities(mut self, value: LlmCapabilitiesObject) -> Self {
        self.capabilities = Some(value);
        self
    }

    pub fn pricing(mut self, value: LlmPricing) -> Self {
        self.pricing = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`LlmModelObject`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](LlmModelObjectBuilder::id)
    /// - [`object`](LlmModelObjectBuilder::object)
    /// - [`created`](LlmModelObjectBuilder::created)
    /// - [`owned_by`](LlmModelObjectBuilder::owned_by)
    /// - [`name`](LlmModelObjectBuilder::name)
    /// - [`description`](LlmModelObjectBuilder::description)
    /// - [`context_length`](LlmModelObjectBuilder::context_length)
    /// - [`max_output_tokens`](LlmModelObjectBuilder::max_output_tokens)
    /// - [`capabilities`](LlmModelObjectBuilder::capabilities)
    /// - [`pricing`](LlmModelObjectBuilder::pricing)
    pub fn build(self) -> Result<LlmModelObject, BuildError> {
        Ok(LlmModelObject {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            object: self.object.ok_or_else(|| BuildError::missing_field("object"))?,
            created: self.created.ok_or_else(|| BuildError::missing_field("created"))?,
            owned_by: self.owned_by.ok_or_else(|| BuildError::missing_field("owned_by"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            description: self.description.ok_or_else(|| BuildError::missing_field("description"))?,
            context_length: self.context_length.ok_or_else(|| BuildError::missing_field("context_length"))?,
            max_output_tokens: self.max_output_tokens.ok_or_else(|| BuildError::missing_field("max_output_tokens"))?,
            capabilities: self.capabilities.ok_or_else(|| BuildError::missing_field("capabilities"))?,
            pricing: self.pricing.ok_or_else(|| BuildError::missing_field("pricing"))?,
        })
    }
}
