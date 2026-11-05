pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Where every instance may connect, beside what the sandbox itself needs
/// to run: a `mode`, or the hosts and IP ranges listed, never both.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxEgress {
    /// `all`, every public address; `default`, PyPI and Hugging Face on 80 and 443; `none`, nothing beyond what the sandbox needs to run. Give this or the rule fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<SandboxEgressMode>,
    /// DNS names the sandbox may connect to, each a name such as `api.github.com` or one with a single leading `*.` label such as `*.githubusercontent.com`; at most 32.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<Vec<String>>,
    /// IP ranges the sandbox may connect to, each in canonical CIDR form such as `203.0.113.0/24`; at most 8. Private, loopback, link-local and other reserved addresses inside a range are never reached, and the create answers 400 for a range wholly inside them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidrs: Option<Vec<String>>,
    /// The TCP ports `hosts` and `cidrs` are reached on, 1 to 65535; 80 and 443 when omitted. At most 8.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<i64>>,
    /// Whether the sandbox also reaches PyPI and Hugging Face on 80 and 443, beside `hosts` and `cidrs`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}

impl SandboxEgress {
    pub fn builder() -> SandboxEgressBuilder {
        <SandboxEgressBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxEgressBuilder {
    mode: Option<SandboxEgressMode>,
    hosts: Option<Vec<String>>,
    cidrs: Option<Vec<String>>,
    ports: Option<Vec<i64>>,
    default: Option<bool>,
}

impl SandboxEgressBuilder {
    pub fn mode(mut self, value: SandboxEgressMode) -> Self {
        self.mode = Some(value);
        self
    }

    pub fn hosts(mut self, value: Vec<String>) -> Self {
        self.hosts = Some(value);
        self
    }

    pub fn cidrs(mut self, value: Vec<String>) -> Self {
        self.cidrs = Some(value);
        self
    }

    pub fn ports(mut self, value: Vec<i64>) -> Self {
        self.ports = Some(value);
        self
    }

    pub fn default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxEgress`].
    pub fn build(self) -> Result<SandboxEgress, BuildError> {
        Ok(SandboxEgress {
            mode: self.mode,
            hosts: self.hosts,
            cidrs: self.cidrs,
            ports: self.ports,
            default: self.default,
        })
    }
}
