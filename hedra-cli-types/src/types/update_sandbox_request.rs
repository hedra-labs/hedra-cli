pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct UpdateSandboxRequest {
    /// When the sandbox stops: a time with its UTC offset, later than now and at most 365 days from now. `null` removes the end time, so the sandbox runs until you stop it or its wallet cannot pay.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub ttl_ends_at: Option<DateTime<FixedOffset>>,
}

impl UpdateSandboxRequest {
    pub fn builder() -> UpdateSandboxRequestBuilder {
        <UpdateSandboxRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct UpdateSandboxRequestBuilder {
    ttl_ends_at: Option<DateTime<FixedOffset>>,
}

impl UpdateSandboxRequestBuilder {
    pub fn ttl_ends_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.ttl_ends_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`UpdateSandboxRequest`].
    pub fn build(self) -> Result<UpdateSandboxRequest, BuildError> {
        Ok(UpdateSandboxRequest {
            ttl_ends_at: self.ttl_ends_at,
        })
    }
}

