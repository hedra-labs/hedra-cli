pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// CPU, memory, scratch disk and shared memory per instance.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxResources {
    /// CPU cores. On a GPU model sold with CPU cores, at most its cores per GPU times the GPU count, which is also the default: 16 per `rtx4090` GPU, 16 per `rtxpro6000` GPU.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<i64>,
    /// Memory, in GB. On a GPU model sold with memory, at most its GB per GPU times the GPU count, which is also the default: 24 per `rtx4090` GPU, 48 per `rtxpro6000` GPU.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_gb: Option<i64>,
    /// Disk space, in GB, for the sandbox: files under `/scratch`, files on the rest of its disk and its stdout and stderr logs count together against it. On a GPU model sold with scratch disk, at most its GB per GPU times the GPU count, which is also the default: 100 per `rtx4090` GPU, 100 per `rtxpro6000` GPU.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scratch_gb: Option<i64>,
    /// Shared memory at `/dev/shm`, in GB. Needs `memory_gb` at least as large. Unset: 8 GB per GPU, capped at half of `memory_gb` when `memory_gb` is set, and at least 1 GB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shm_gb: Option<i64>,
}

impl SandboxResources {
    pub fn builder() -> SandboxResourcesBuilder {
        <SandboxResourcesBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxResourcesBuilder {
    cpu: Option<i64>,
    memory_gb: Option<i64>,
    scratch_gb: Option<i64>,
    shm_gb: Option<i64>,
}

impl SandboxResourcesBuilder {
    pub fn cpu(mut self, value: i64) -> Self {
        self.cpu = Some(value);
        self
    }

    pub fn memory_gb(mut self, value: i64) -> Self {
        self.memory_gb = Some(value);
        self
    }

    pub fn scratch_gb(mut self, value: i64) -> Self {
        self.scratch_gb = Some(value);
        self
    }

    pub fn shm_gb(mut self, value: i64) -> Self {
        self.shm_gb = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxResources`].
    pub fn build(self) -> Result<SandboxResources, BuildError> {
        Ok(SandboxResources {
            cpu: self.cpu,
            memory_gb: self.memory_gb,
            scratch_gb: self.scratch_gb,
            shm_gb: self.shm_gb,
        })
    }
}
