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
    #[serde(default)]
    pub sampling_parameters: Vec<String>,
    #[serde(default)]
    pub reasoning_efforts: Vec<String>,
    #[serde(default)]
    pub response_format_types: Vec<String>,
    #[serde(default)]
    pub parallel_tool_calls: bool,
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
    sampling_parameters: Option<Vec<String>>,
    reasoning_efforts: Option<Vec<String>>,
    response_format_types: Option<Vec<String>>,
    parallel_tool_calls: Option<bool>,
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

    pub fn sampling_parameters(mut self, value: Vec<String>) -> Self {
        self.sampling_parameters = Some(value);
        self
    }

    pub fn reasoning_efforts(mut self, value: Vec<String>) -> Self {
        self.reasoning_efforts = Some(value);
        self
    }

    pub fn response_format_types(mut self, value: Vec<String>) -> Self {
        self.response_format_types = Some(value);
        self
    }

    pub fn parallel_tool_calls(mut self, value: bool) -> Self {
        self.parallel_tool_calls = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`LlmCapabilitiesObject`].
    /// This method will fail if any of the following fields are not set:
    /// - [`tools`](LlmCapabilitiesObjectBuilder::tools)
    /// - [`response_format`](LlmCapabilitiesObjectBuilder::response_format)
    /// - [`vision`](LlmCapabilitiesObjectBuilder::vision)
    /// - [`reasoning`](LlmCapabilitiesObjectBuilder::reasoning)
    /// - [`sampling_parameters`](LlmCapabilitiesObjectBuilder::sampling_parameters)
    /// - [`reasoning_efforts`](LlmCapabilitiesObjectBuilder::reasoning_efforts)
    /// - [`response_format_types`](LlmCapabilitiesObjectBuilder::response_format_types)
    /// - [`parallel_tool_calls`](LlmCapabilitiesObjectBuilder::parallel_tool_calls)
    pub fn build(self) -> Result<LlmCapabilitiesObject, BuildError> {
        Ok(LlmCapabilitiesObject {
            tools: self.tools.ok_or_else(|| BuildError::missing_field("tools"))?,
            response_format: self.response_format.ok_or_else(|| BuildError::missing_field("response_format"))?,
            vision: self.vision.ok_or_else(|| BuildError::missing_field("vision"))?,
            reasoning: self.reasoning.ok_or_else(|| BuildError::missing_field("reasoning"))?,
            sampling_parameters: self.sampling_parameters.ok_or_else(|| BuildError::missing_field("sampling_parameters"))?,
            reasoning_efforts: self.reasoning_efforts.ok_or_else(|| BuildError::missing_field("reasoning_efforts"))?,
            response_format_types: self.response_format_types.ok_or_else(|| BuildError::missing_field("response_format_types"))?,
            parallel_tool_calls: self.parallel_tool_calls.ok_or_else(|| BuildError::missing_field("parallel_tool_calls"))?,
        })
    }
}
