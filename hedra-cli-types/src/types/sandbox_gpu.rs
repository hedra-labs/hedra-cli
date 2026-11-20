pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A GPU model your workspace can run.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SandboxGpu {
    /// The GPU model, as `gpu` names it (`h100`).
    #[serde(default)]
    pub gpu_model: String,
    /// The GPU's maker, `nvidia` or `amd`. An image built for one vendor's GPUs, such as a CUDA build for `nvidia`, does not run on the other's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Memory per GPU, in GB.
    #[serde(default)]
    pub gpu_memory_gb: i64,
    /// One GPU-hour at your workspace's price, after its discount.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub usd_per_gpu_hour: f64,
    /// Whether one more sandbox of this model fits in the GPUs free right now in the pools that serve it, shared with other workspaces; null when that is unknown, such as while a pool has not reported its GPUs recently. `false` means a new sandbox may wait for GPUs to free up. `true` does not promise an immediate start: the sandbox is placed in one pool, a machine may go out of service, a request for several GPUs may need more than are free or than one machine holds, and sandboxes already waiting take free GPUs first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
    /// The GPU counts per instance that the pools serving this model offer, ascending, as `gpu` takes it after the colon; null when unknown. The list is neither complete nor a promise: a count not listed can still place, up to a pool's cap, and a listed count can wait for a machine with that many GPUs free or fail to place. Your workspace's GPU quota (`GET /sandboxes/quota`) can still refuse a count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_counts: Option<Vec<i64>>,
    /// Whether a sandbox of this model created with `ports` gets an `internet_url`, an address reachable from the public internet. `false` means its only address is `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internet: Option<bool>,
    /// Whether a sandbox of this model may name `secrets` or reference a saved Secret from `env` (`{"ref": NAME}`); null when that is unknown. `false` means a create that does either can be refused. A reference to a saved Config and a literal `env` value work either way.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets_supported: Option<bool>,
}

impl SandboxGpu {
    pub fn builder() -> SandboxGpuBuilder {
        <SandboxGpuBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxGpuBuilder {
    gpu_model: Option<String>,
    vendor: Option<String>,
    gpu_memory_gb: Option<i64>,
    usd_per_gpu_hour: Option<f64>,
    available: Option<bool>,
    gpu_counts: Option<Vec<i64>>,
    internet: Option<bool>,
    secrets_supported: Option<bool>,
}

impl SandboxGpuBuilder {
    pub fn gpu_model(mut self, value: impl Into<String>) -> Self {
        self.gpu_model = Some(value.into());
        self
    }

    pub fn vendor(mut self, value: impl Into<String>) -> Self {
        self.vendor = Some(value.into());
        self
    }

    pub fn gpu_memory_gb(mut self, value: i64) -> Self {
        self.gpu_memory_gb = Some(value);
        self
    }

    pub fn usd_per_gpu_hour(mut self, value: f64) -> Self {
        self.usd_per_gpu_hour = Some(value);
        self
    }

    pub fn available(mut self, value: bool) -> Self {
        self.available = Some(value);
        self
    }

    pub fn gpu_counts(mut self, value: Vec<i64>) -> Self {
        self.gpu_counts = Some(value);
        self
    }

    pub fn internet(mut self, value: bool) -> Self {
        self.internet = Some(value);
        self
    }

    pub fn secrets_supported(mut self, value: bool) -> Self {
        self.secrets_supported = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxGpu`].
    /// This method will fail if any of the following fields are not set:
    /// - [`gpu_model`](SandboxGpuBuilder::gpu_model)
    /// - [`gpu_memory_gb`](SandboxGpuBuilder::gpu_memory_gb)
    /// - [`usd_per_gpu_hour`](SandboxGpuBuilder::usd_per_gpu_hour)
    pub fn build(self) -> Result<SandboxGpu, BuildError> {
        Ok(SandboxGpu {
            gpu_model: self.gpu_model.ok_or_else(|| BuildError::missing_field("gpu_model"))?,
            vendor: self.vendor,
            gpu_memory_gb: self.gpu_memory_gb.ok_or_else(|| BuildError::missing_field("gpu_memory_gb"))?,
            usd_per_gpu_hour: self.usd_per_gpu_hour.ok_or_else(|| BuildError::missing_field("usd_per_gpu_hour"))?,
            available: self.available,
            gpu_counts: self.gpu_counts,
            internet: self.internet,
            secrets_supported: self.secrets_supported,
        })
    }
}
