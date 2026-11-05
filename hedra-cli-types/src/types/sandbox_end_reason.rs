pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Why a sandbox ended, or, on an `active` one, that its last lease was
/// refused.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxEndReason {
    CallerStopped,
    TtlEnded,
    Exited,
    InstanceFailed,
    Unpaid,
    LeaseLapsed,
    AccessRemoved,
    IdleTtlEnded,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxEndReason {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::CallerStopped => serializer.serialize_str("caller_stopped"),
            Self::TtlEnded => serializer.serialize_str("ttl_ended"),
            Self::Exited => serializer.serialize_str("exited"),
            Self::InstanceFailed => serializer.serialize_str("instance_failed"),
            Self::Unpaid => serializer.serialize_str("unpaid"),
            Self::LeaseLapsed => serializer.serialize_str("lease_lapsed"),
            Self::AccessRemoved => serializer.serialize_str("access_removed"),
            Self::IdleTtlEnded => serializer.serialize_str("idle_ttl_ended"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxEndReason {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "caller_stopped" => Ok(Self::CallerStopped),
            "ttl_ended" => Ok(Self::TtlEnded),
            "exited" => Ok(Self::Exited),
            "instance_failed" => Ok(Self::InstanceFailed),
            "unpaid" => Ok(Self::Unpaid),
            "lease_lapsed" => Ok(Self::LeaseLapsed),
            "access_removed" => Ok(Self::AccessRemoved),
            "idle_ttl_ended" => Ok(Self::IdleTtlEnded),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxEndReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CallerStopped => write!(f, "caller_stopped"),
            Self::TtlEnded => write!(f, "ttl_ended"),
            Self::Exited => write!(f, "exited"),
            Self::InstanceFailed => write!(f, "instance_failed"),
            Self::Unpaid => write!(f, "unpaid"),
            Self::LeaseLapsed => write!(f, "lease_lapsed"),
            Self::AccessRemoved => write!(f, "access_removed"),
            Self::IdleTtlEnded => write!(f, "idle_ttl_ended"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
