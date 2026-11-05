pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One GPU sandbox.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sandbox {
    /// The sandbox's id (`sandbox_<uuid>`).
    #[serde(default)]
    pub id: String,
    /// The sandbox's name, as created.
    #[serde(default)]
    pub name: String,
    /// The request's `gpu`.
    #[serde(default)]
    pub gpu: String,
    /// The `gpu`'s GPU model.
    #[serde(default)]
    pub gpu_model: String,
    /// How many instances.
    #[serde(default)]
    pub replicas: i64,
    pub status: SandboxStatus,
    /// Why the sandbox ended. On an `active` sandbox, set only while its last lease is refused for funds (`unpaid`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_reason: Option<SandboxEndReason>,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    /// The earliest `started_at` of its instances; null on the list and before any instance starts.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub started_at: Option<DateTime<FixedOffset>>,
    /// When the last instance stopped; null until then.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub ended_at: Option<DateTime<FixedOffset>>,
    /// When the sandbox's lifetime ends: `created_at` plus `ttl_seconds`; null for a sandbox created without `ttl_seconds`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub ttl_ends_at: Option<DateTime<FixedOffset>>,
    /// The keys of `env`, never its values.
    #[serde(default)]
    pub env_keys: Vec<String>,
    /// The image the sandbox runs, as created. Set wherever `instances` is and the sandbox has an instance; null on the list, on the create answer, and while it has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// The `command` the sandbox runs, as created and returned as given, so a secret belongs in `env` rather than here; null when `image` is, when the image's own `ENTRYPOINT`/`CMD` runs, and for an `idle` sandbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// One entry per replica; null on the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<SandboxInstance>>,
    /// The address of the port in `ports`, the same for every replica; call it with a token from `POST /sandboxes/{sandbox}/endpoint-token`. Set wherever `instances` is; null on the list, for a sandbox created without `ports`, and while its pool serves no address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub billing: SandboxBillingSummary,
}

impl Sandbox {
    pub fn builder() -> SandboxBuilder {
        <SandboxBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxBuilder {
    id: Option<String>,
    name: Option<String>,
    gpu: Option<String>,
    gpu_model: Option<String>,
    replicas: Option<i64>,
    status: Option<SandboxStatus>,
    end_reason: Option<SandboxEndReason>,
    created_at: Option<DateTime<FixedOffset>>,
    started_at: Option<DateTime<FixedOffset>>,
    ended_at: Option<DateTime<FixedOffset>>,
    ttl_ends_at: Option<DateTime<FixedOffset>>,
    env_keys: Option<Vec<String>>,
    image: Option<String>,
    command: Option<Vec<String>>,
    instances: Option<Vec<SandboxInstance>>,
    url: Option<String>,
    billing: Option<SandboxBillingSummary>,
}

impl SandboxBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn gpu(mut self, value: impl Into<String>) -> Self {
        self.gpu = Some(value.into());
        self
    }

    pub fn gpu_model(mut self, value: impl Into<String>) -> Self {
        self.gpu_model = Some(value.into());
        self
    }

    pub fn replicas(mut self, value: i64) -> Self {
        self.replicas = Some(value);
        self
    }

    pub fn status(mut self, value: SandboxStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn end_reason(mut self, value: SandboxEndReason) -> Self {
        self.end_reason = Some(value);
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn started_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.started_at = Some(value);
        self
    }

    pub fn ended_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.ended_at = Some(value);
        self
    }

    pub fn ttl_ends_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.ttl_ends_at = Some(value);
        self
    }

    pub fn env_keys(mut self, value: Vec<String>) -> Self {
        self.env_keys = Some(value);
        self
    }

    pub fn image(mut self, value: impl Into<String>) -> Self {
        self.image = Some(value.into());
        self
    }

    pub fn command(mut self, value: Vec<String>) -> Self {
        self.command = Some(value);
        self
    }

    pub fn instances(mut self, value: Vec<SandboxInstance>) -> Self {
        self.instances = Some(value);
        self
    }

    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn billing(mut self, value: SandboxBillingSummary) -> Self {
        self.billing = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`Sandbox`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](SandboxBuilder::id)
    /// - [`name`](SandboxBuilder::name)
    /// - [`gpu`](SandboxBuilder::gpu)
    /// - [`gpu_model`](SandboxBuilder::gpu_model)
    /// - [`replicas`](SandboxBuilder::replicas)
    /// - [`status`](SandboxBuilder::status)
    /// - [`created_at`](SandboxBuilder::created_at)
    /// - [`env_keys`](SandboxBuilder::env_keys)
    /// - [`billing`](SandboxBuilder::billing)
    pub fn build(self) -> Result<Sandbox, BuildError> {
        Ok(Sandbox {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            gpu: self.gpu.ok_or_else(|| BuildError::missing_field("gpu"))?,
            gpu_model: self.gpu_model.ok_or_else(|| BuildError::missing_field("gpu_model"))?,
            replicas: self.replicas.ok_or_else(|| BuildError::missing_field("replicas"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            end_reason: self.end_reason,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            started_at: self.started_at,
            ended_at: self.ended_at,
            ttl_ends_at: self.ttl_ends_at,
            env_keys: self.env_keys.ok_or_else(|| BuildError::missing_field("env_keys"))?,
            image: self.image,
            command: self.command,
            instances: self.instances,
            url: self.url,
            billing: self.billing.ok_or_else(|| BuildError::missing_field("billing"))?,
        })
    }
}
