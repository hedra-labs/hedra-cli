pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// What this model accepts and returns.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct LlmCapabilitiesObject {
    #[serde(default)]
    pub tools: bool,
    #[serde(default)]
    pub response_format: bool,
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub reasoning: bool,
}

impl LlmCapabilitiesObject {
    pub fn builder() -> LlmCapabilitiesObjectBuilder {
        <LlmCapabilitiesObjectBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LlmCapabilitiesObjectBuilder {
    tools: Option<bool>,
    response_format: Option<bool>,
    vision: Option<bool>,
    reasoning: Option<bool>,
}

impl LlmCapabilitiesObjectBuilder {
    pub fn tools(mut self, value: bool) -> Self {
        self.tools = Some(value);
        self
    }

    pub fn response_format(mut self, value: bool) -> Self {
        self.response_format = Some(value);
        self
    }

    pub fn vision(mut self, value: bool) -> Self {
        self.vision = Some(value);
        self
    }

    pub fn reasoning(mut self, value: bool) -> Self {
        self.reasoning = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`LlmCapabilitiesObject`].
    /// This method will fail if any of the following fields are not set:
    /// - [`tools`](LlmCapabilitiesObjectBuilder::tools)
    /// - [`response_format`](LlmCapabilitiesObjectBuilder::response_format)
    /// - [`vision`](LlmCapabilitiesObjectBuilder::vision)
    /// - [`reasoning`](LlmCapabilitiesObjectBuilder::reasoning)
    pub fn build(self) -> Result<LlmCapabilitiesObject, BuildError> {
        Ok(LlmCapabilitiesObject {
            tools: self.tools.ok_or_else(|| BuildError::missing_field("tools"))?,
            response_format: self.response_format.ok_or_else(|| BuildError::missing_field("response_format"))?,
            vision: self.vision.ok_or_else(|| BuildError::missing_field("vision"))?,
            reasoning: self.reasoning.ok_or_else(|| BuildError::missing_field("reasoning"))?,
        })
    }
}
