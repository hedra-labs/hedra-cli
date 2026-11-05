pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Where a sandbox is in its life; stored in ``sandbox_billings.status``.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxStatus {
    Creating,
    Active,
    Stopping,
    Ended,
    Failed,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Creating => serializer.serialize_str("creating"),
            Self::Active => serializer.serialize_str("active"),
            Self::Stopping => serializer.serialize_str("stopping"),
            Self::Ended => serializer.serialize_str("ended"),
            Self::Failed => serializer.serialize_str("failed"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "creating" => Ok(Self::Creating),
            "active" => Ok(Self::Active),
            "stopping" => Ok(Self::Stopping),
            "ended" => Ok(Self::Ended),
            "failed" => Ok(Self::Failed),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creating => write!(f, "creating"),
            Self::Active => write!(f, "active"),
            Self::Stopping => write!(f, "stopping"),
            Self::Ended => write!(f, "ended"),
            Self::Failed => write!(f, "failed"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
