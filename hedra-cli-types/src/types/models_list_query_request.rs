pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for list
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ModelsListQueryRequest {
    /// Only models with this modality, matching `modality` on each returned model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<Modality>,
}

impl ModelsListQueryRequest {
    pub fn builder() -> ModelsListQueryRequestBuilder {
        <ModelsListQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ModelsListQueryRequestBuilder {
    modality: Option<Modality>,
}

impl ModelsListQueryRequestBuilder {
    pub fn modality(mut self, value: Modality) -> Self {
        self.modality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ModelsListQueryRequest`].
    pub fn build(self) -> Result<ModelsListQueryRequest, BuildError> {
        Ok(ModelsListQueryRequest {
            modality: self.modality,
        })
    }
}

