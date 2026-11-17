pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CreateSandboxRequest {
    /// The sandbox's name: lowercase letters, digits and hyphens, starting and ending with a letter or digit, at most 32 characters, and not `environment-variables`, `gpus`, `quota`, `secrets`, `signing-key`. Unique among your creating, active and stopping sandboxes, and usable in place of the id in a sandbox's path.
    #[serde(default)]
    pub name: String,
    /// The image reference to run.
    #[serde(default)]
    pub image: String,
    /// A GPU model and a GPU count per instance, `<gpu model>:<count>` such as `h100:1`. The count is 1 to 99; `GET /sandboxes/gpus` lists the GPU models.
    #[serde(default)]
    pub gpu: String,
    /// How many instances, at most your workspace's sandbox quota.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<i64>,
    /// CPU, memory, scratch disk and shared memory per instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<SandboxResources>,
    /// What runs in place of the image's entrypoint; at most 16384 bytes written as JSON. Absent runs the image's `ENTRYPOINT`/`CMD`. The main process's stdin stays open, as under `docker run -i`, so a shell or an interpreter waits for input. The sandbox ends when the main process exits unless `restart` restarts it; set `idle` for a sandbox with no main process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// Run no main process: the sandbox starts with nothing running and stays up for sessions until you stop it, `ttl_seconds` ends or it is inactive for `stop_after_inactive_seconds`. Not with `command`; `restart` has no effect. The image's entrypoint is not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle: Option<bool>,
    /// When the command is restarted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<SandboxRestartPolicy>,
    /// Environment variables for every instance, by name. Each value is a string, or `{"ref": NAME}` naming a saved workspace environment variable, Config or Secret, whose value each instance reads when it starts; at most 128 references. At most 32768 bytes written as JSON, no empty key or key containing `=`, and no key starting `SANDBOX_`, which the sandbox reserves. A reference's key also follows the variable name rule and is not one of the names `secrets` lists. Answers return the keys only, as `env_keys`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, CreateSandboxRequestEnvValue>>,
    /// The sandbox's lifetime in seconds, counted from create, 120 to 86,400 (24 hours). Required; every instance is stopped when it ends. Longer reservations require a sales contract with Hedra; contact sales@hedra.com.
    #[serde(default)]
    pub ttl_seconds: i64,
    /// Stop the sandbox, with `end_reason` `inactive`, once it has had no activity for this many seconds, 120 to 31,536,000. Activity is a request to its `url` or `internet_url`, until its response ends, and an exec or shell session, from its start until its process exits. The sandbox's command, a process left running after its session ends, a port-forward, an attach, a request refused because its endpoint token is missing or invalid, and a request answered 503 before the server first serves are not activity. The count starts when each instance starts, so set a value longer than your server takes to start serving. With `auth` `none`, a request from any caller counts, so any caller can keep the sandbox running until you stop it or its end time passes. With more than one replica, the instances stop together once every one is inactive. The stop can land a minute or so after the limit. Leave it out for no limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_after_inactive_seconds: Option<i64>,
    /// The port to serve at the sandbox's `url`; at most 1. With `ports`, `name` never contains `--`. A recreate under the same `name` on another GPU model can get a new `url`; endpoint tokens minted for the name keep working.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<SandboxPort>>,
    /// The names of workspace secrets every instance receives, each as an environment variable of its own name; at most 32. `GET /sandboxes/secrets` lists the names your workspace holds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<String>>,
    /// Where every instance may connect: `{"mode": "all"}`, the default, or another `mode`, or the hosts and IP ranges listed. Beyond what the sandbox needs to run, no form reaches a private, loopback or link-local address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egress: Option<SandboxEgress>,
}

impl CreateSandboxRequest {
    pub fn builder() -> CreateSandboxRequestBuilder {
        <CreateSandboxRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct CreateSandboxRequestBuilder {
    name: Option<String>,
    image: Option<String>,
    gpu: Option<String>,
    replicas: Option<i64>,
    resources: Option<SandboxResources>,
    command: Option<Vec<String>>,
    idle: Option<bool>,
    restart: Option<SandboxRestartPolicy>,
    env: Option<HashMap<String, CreateSandboxRequestEnvValue>>,
    ttl_seconds: Option<i64>,
    stop_after_inactive_seconds: Option<i64>,
    ports: Option<Vec<SandboxPort>>,
    secrets: Option<Vec<String>>,
    egress: Option<SandboxEgress>,
}

impl CreateSandboxRequestBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn image(mut self, value: impl Into<String>) -> Self {
        self.image = Some(value.into());
        self
    }

    pub fn gpu(mut self, value: impl Into<String>) -> Self {
        self.gpu = Some(value.into());
        self
    }

    pub fn replicas(mut self, value: i64) -> Self {
        self.replicas = Some(value);
        self
    }

    pub fn resources(mut self, value: SandboxResources) -> Self {
        self.resources = Some(value);
        self
    }

    pub fn command(mut self, value: Vec<String>) -> Self {
        self.command = Some(value);
        self
    }

    pub fn idle(mut self, value: bool) -> Self {
        self.idle = Some(value);
        self
    }

    pub fn restart(mut self, value: SandboxRestartPolicy) -> Self {
        self.restart = Some(value);
        self
    }

    pub fn env(mut self, value: HashMap<String, CreateSandboxRequestEnvValue>) -> Self {
        self.env = Some(value);
        self
    }

    pub fn ttl_seconds(mut self, value: i64) -> Self {
        self.ttl_seconds = Some(value);
        self
    }

    pub fn stop_after_inactive_seconds(mut self, value: i64) -> Self {
        self.stop_after_inactive_seconds = Some(value);
        self
    }

    pub fn ports(mut self, value: Vec<SandboxPort>) -> Self {
        self.ports = Some(value);
        self
    }

    pub fn secrets(mut self, value: Vec<String>) -> Self {
        self.secrets = Some(value);
        self
    }

    pub fn egress(mut self, value: SandboxEgress) -> Self {
        self.egress = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`CreateSandboxRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](CreateSandboxRequestBuilder::name)
    /// - [`image`](CreateSandboxRequestBuilder::image)
    /// - [`gpu`](CreateSandboxRequestBuilder::gpu)
    /// - [`ttl_seconds`](CreateSandboxRequestBuilder::ttl_seconds)
    pub fn build(self) -> Result<CreateSandboxRequest, BuildError> {
        Ok(CreateSandboxRequest {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            image: self.image.ok_or_else(|| BuildError::missing_field("image"))?,
            gpu: self.gpu.ok_or_else(|| BuildError::missing_field("gpu"))?,
            replicas: self.replicas,
            resources: self.resources,
            command: self.command,
            idle: self.idle,
            restart: self.restart,
            env: self.env,
            ttl_seconds: self.ttl_seconds.ok_or_else(|| BuildError::missing_field("ttl_seconds"))?,
            stop_after_inactive_seconds: self.stop_after_inactive_seconds,
            ports: self.ports,
            secrets: self.secrets,
            egress: self.egress,
        })
    }
}

