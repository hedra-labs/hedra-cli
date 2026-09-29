pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyVeedFabric10 {
    pub input: InputVeedFabric10,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyVeedFabric10 {
    pub fn builder() -> SubmitBodyVeedFabric10Builder {
        <SubmitBodyVeedFabric10Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyVeedFabric10Builder {
    input: Option<InputVeedFabric10>,
    webhook: Option<String>,
}

impl SubmitBodyVeedFabric10Builder {
    pub fn input(mut self, value: InputVeedFabric10) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyVeedFabric10`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyVeedFabric10Builder::input)
    pub fn build(self) -> Result<SubmitBodyVeedFabric10, BuildError> {
        Ok(SubmitBodyVeedFabric10 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

