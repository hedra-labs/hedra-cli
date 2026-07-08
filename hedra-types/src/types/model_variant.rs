pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ModelVariant {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<HashMap<String, serde_json::Value>>,
}

impl ModelVariant {
    pub fn builder() -> ModelVariantBuilder {
        <ModelVariantBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ModelVariantBuilder {
    id: Option<String>,
    mode: Option<String>,
    input_schema: Option<HashMap<String, serde_json::Value>>,
    output_schema: Option<HashMap<String, serde_json::Value>>,
}

impl ModelVariantBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn mode(mut self, value: impl Into<String>) -> Self {
        self.mode = Some(value.into());
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

    /// Consumes the builder and constructs a [`ModelVariant`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](ModelVariantBuilder::id)
    /// - [`mode`](ModelVariantBuilder::mode)
    pub fn build(self) -> Result<ModelVariant, BuildError> {
        Ok(ModelVariant {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            mode: self.mode.ok_or_else(|| BuildError::missing_field("mode"))?,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
        })
    }
}
