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
    /// The request's `gpu`; null for a sandbox with no GPU.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<String>,
    /// The `gpu`'s GPU model; null for a sandbox with no GPU.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_model: Option<String>,
    /// How many instances.
    #[serde(default)]
    pub replicas: i64,
    pub status: SandboxStatus,
    /// How the sandbox is paid for: `lease` from the API wallet, or `capacity_reservation` by the workspace's capacity reservation. Fixed when the sandbox is created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_mode: Option<SandboxBillingMode>,
    /// Why the sandbox ended. On an `active` sandbox, set only while its last lease is refused for funds (`unpaid`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_reason: Option<SandboxEndReason>,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    /// Who created the sandbox: you, another member of your workspace, or its service key. Set on every answer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<SandboxCreator>,
    /// Who stopped the sandbox when `end_reason` is `caller_stopped`: you, another member of your workspace, or its service key. Null for every other `end_reason`, and for a sandbox stopped before Hedra recorded who stopped it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped_by: Option<SandboxCreator>,
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
    /// When the sandbox stops: `created_at` plus `ttl_seconds`, or the time last set with `PATCH /v3/sandboxes/{sandbox}`; null for no end time or when `billing_mode` is `capacity_reservation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub ttl_ends_at: Option<DateTime<FixedOffset>>,
    /// The keys of `env`, never its values.
    #[serde(default)]
    pub env_keys: Vec<String>,
    /// The image the sandbox runs, as created. Set wherever `instances` is and the sandbox has an instance; null on the list, on the create answer, while it has none, and once the owner has created a later sandbox of its name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// The `command` the sandbox runs, as created and returned as given, so a secret belongs in `env` rather than here; null when `image` is, when the image's own `ENTRYPOINT`/`CMD` runs, and for an `idle` sandbox. Every member of the workspace reads it, so a secret in it is a secret shared with the workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// One entry per replica; null on the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<SandboxInstance>>,
    /// The address of the port in `ports`, the same for every replica. With `auth` `token`, each request also carries a token from `POST /sandboxes/{sandbox}/endpoint-token`. Set wherever `instances` is; null on the list, for a sandbox created without `ports`, while its compute pool serves no address, and after any later create of the same sandbox name in the same workspace by the same owner, including a failed create. Recreates with `ports` in that scope keep the routed URL while the compute pool's gateway host stays the same. An earlier endpoint token or `browser_url` still reaches the replacement until its original `expires_at` or signing-key rotation. Stopping, recreating or changing the image does not revoke the endpoint token; another sandbox name isolates the replacement without revoking the earlier endpoint token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The address of the port in `ports` from the public internet, beside `url`; the port's `auth` applies to both, and the same token works on both. Null wherever `url` is null, for a sandbox created while its environment served no public addresses, and while its compute pool serves none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internet_url: Option<String>,
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
    billing_mode: Option<SandboxBillingMode>,
    end_reason: Option<SandboxEndReason>,
    created_at: Option<DateTime<FixedOffset>>,
    created_by: Option<SandboxCreator>,
    stopped_by: Option<SandboxCreator>,
    started_at: Option<DateTime<FixedOffset>>,
    ended_at: Option<DateTime<FixedOffset>>,
    ttl_ends_at: Option<DateTime<FixedOffset>>,
    env_keys: Option<Vec<String>>,
    image: Option<String>,
    command: Option<Vec<String>>,
    instances: Option<Vec<SandboxInstance>>,
    url: Option<String>,
    internet_url: Option<String>,
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

    pub fn billing_mode(mut self, value: SandboxBillingMode) -> Self {
        self.billing_mode = Some(value);
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

    pub fn created_by(mut self, value: SandboxCreator) -> Self {
        self.created_by = Some(value);
        self
    }

    pub fn stopped_by(mut self, value: SandboxCreator) -> Self {
        self.stopped_by = Some(value);
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

    pub fn internet_url(mut self, value: impl Into<String>) -> Self {
        self.internet_url = Some(value.into());
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
    /// - [`replicas`](SandboxBuilder::replicas)
    /// - [`status`](SandboxBuilder::status)
    /// - [`created_at`](SandboxBuilder::created_at)
    /// - [`env_keys`](SandboxBuilder::env_keys)
    /// - [`billing`](SandboxBuilder::billing)
    pub fn build(self) -> Result<Sandbox, BuildError> {
        Ok(Sandbox {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            gpu: self.gpu,
            gpu_model: self.gpu_model,
            replicas: self.replicas.ok_or_else(|| BuildError::missing_field("replicas"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            billing_mode: self.billing_mode,
            end_reason: self.end_reason,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            created_by: self.created_by,
            stopped_by: self.stopped_by,
            started_at: self.started_at,
            ended_at: self.ended_at,
            ttl_ends_at: self.ttl_ends_at,
            env_keys: self.env_keys.ok_or_else(|| BuildError::missing_field("env_keys"))?,
            image: self.image,
            command: self.command,
            instances: self.instances,
            url: self.url,
            internet_url: self.internet_url,
            billing: self.billing.ok_or_else(|| BuildError::missing_field("billing"))?,
        })
    }
}
