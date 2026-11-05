pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// How much of your workspace's GPU quota is left.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxQuota {
    /// GPUs left under your workspace's GPU quota: the quota less the GPUs its `creating`, `active` and `stopping` sandboxes were created with, every member's included. A create can still answer 429 with GPUs left, because your workspace's sandbox quota also limits its running instances, every replica counted. A sandbox counts at its full `replicas` until Hedra records it ended: usually a minute or two after a stop, the end of `ttl_seconds` or its last replica's exit, and for as long as any replica still runs.
    #[serde(default)]
    pub gpu_quota_remaining: i64,
}

impl SandboxQuota {
    pub fn builder() -> SandboxQuotaBuilder {
        <SandboxQuotaBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxQuotaBuilder {
    gpu_quota_remaining: Option<i64>,
}

impl SandboxQuotaBuilder {
    pub fn gpu_quota_remaining(mut self, value: i64) -> Self {
        self.gpu_quota_remaining = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxQuota`].
    /// This method will fail if any of the following fields are not set:
    /// - [`gpu_quota_remaining`](SandboxQuotaBuilder::gpu_quota_remaining)
    pub fn build(self) -> Result<SandboxQuota, BuildError> {
        Ok(SandboxQuota {
            gpu_quota_remaining: self.gpu_quota_remaining.ok_or_else(|| BuildError::missing_field("gpu_quota_remaining"))?,
        })
    }
}
