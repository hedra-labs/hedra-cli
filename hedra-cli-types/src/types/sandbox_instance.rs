pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One instance of a sandbox: one replica.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SandboxInstance {
    /// The replica's number, from 0.
    #[serde(default)]
    pub replica: i64,
    pub status: SandboxInstanceStatus,
    /// When the instance started running; null while it starts.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub started_at: Option<DateTime<FixedOffset>>,
    /// When the instance stopped; null while it runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub stopped_at: Option<DateTime<FixedOffset>>,
    /// Why the instance failed; null unless `status` is `failed`. `platform_error`: the platform lost the instance or could not stage it; creating the sandbox again may succeed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<SandboxFailureReason>,
}

impl SandboxInstance {
    pub fn builder() -> SandboxInstanceBuilder {
        <SandboxInstanceBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxInstanceBuilder {
    replica: Option<i64>,
    status: Option<SandboxInstanceStatus>,
    started_at: Option<DateTime<FixedOffset>>,
    stopped_at: Option<DateTime<FixedOffset>>,
    failure_reason: Option<SandboxFailureReason>,
}

impl SandboxInstanceBuilder {
    pub fn replica(mut self, value: i64) -> Self {
        self.replica = Some(value);
        self
    }

    pub fn status(mut self, value: SandboxInstanceStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn started_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.started_at = Some(value);
        self
    }

    pub fn stopped_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.stopped_at = Some(value);
        self
    }

    pub fn failure_reason(mut self, value: SandboxFailureReason) -> Self {
        self.failure_reason = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxInstance`].
    /// This method will fail if any of the following fields are not set:
    /// - [`replica`](SandboxInstanceBuilder::replica)
    /// - [`status`](SandboxInstanceBuilder::status)
    pub fn build(self) -> Result<SandboxInstance, BuildError> {
        Ok(SandboxInstance {
            replica: self.replica.ok_or_else(|| BuildError::missing_field("replica"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            started_at: self.started_at,
            stopped_at: self.stopped_at,
            failure_reason: self.failure_reason,
        })
    }
}
