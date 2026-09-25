pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmModelList {
    pub object: LlmModelListObject,
    #[serde(default)]
    pub data: Vec<LlmModelObject>,
}

impl LlmModelList {
    pub fn builder() -> LlmModelListBuilder {
        <LlmModelListBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LlmModelListBuilder {
    object: Option<LlmModelListObject>,
    data: Option<Vec<LlmModelObject>>,
}

impl LlmModelListBuilder {
    pub fn object(mut self, value: LlmModelListObject) -> Self {
        self.object = Some(value);
        self
    }

    pub fn data(mut self, value: Vec<LlmModelObject>) -> Self {
        self.data = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`LlmModelList`].
    /// This method will fail if any of the following fields are not set:
    /// - [`object`](LlmModelListBuilder::object)
    /// - [`data`](LlmModelListBuilder::data)
    pub fn build(self) -> Result<LlmModelList, BuildError> {
        Ok(LlmModelList {
            object: self.object.ok_or_else(|| BuildError::missing_field("object"))?,
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
        })
    }
}
