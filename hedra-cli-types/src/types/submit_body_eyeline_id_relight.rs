pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyEyelineIdRelight {
    pub input: InputEyelineIdRelight,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyEyelineIdRelight {
    pub fn builder() -> SubmitBodyEyelineIdRelightBuilder {
        <SubmitBodyEyelineIdRelightBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyEyelineIdRelightBuilder {
    input: Option<InputEyelineIdRelight>,
    webhook: Option<String>,
}

impl SubmitBodyEyelineIdRelightBuilder {
    pub fn input(mut self, value: InputEyelineIdRelight) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyEyelineIdRelight`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyEyelineIdRelightBuilder::input)
    pub fn build(self) -> Result<SubmitBodyEyelineIdRelight, BuildError> {
        Ok(SubmitBodyEyelineIdRelight {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}

