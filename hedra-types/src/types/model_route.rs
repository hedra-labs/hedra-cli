pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ModelRoute {
    #[serde(default)]
    pub when: String,
    #[serde(default)]
    pub variant: String,
}

impl ModelRoute {
    pub fn builder() -> ModelRouteBuilder {
        <ModelRouteBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ModelRouteBuilder {
    when: Option<String>,
    variant: Option<String>,
}

impl ModelRouteBuilder {
    pub fn when(mut self, value: impl Into<String>) -> Self {
        self.when = Some(value.into());
        self
    }

    pub fn variant(mut self, value: impl Into<String>) -> Self {
        self.variant = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ModelRoute`].
    /// This method will fail if any of the following fields are not set:
    /// - [`when`](ModelRouteBuilder::when)
    /// - [`variant`](ModelRouteBuilder::variant)
    pub fn build(self) -> Result<ModelRoute, BuildError> {
        Ok(ModelRoute {
            when: self.when.ok_or_else(|| BuildError::missing_field("when"))?,
            variant: self.variant.ok_or_else(|| BuildError::missing_field("variant"))?,
        })
    }
}
