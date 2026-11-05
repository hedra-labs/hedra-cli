pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The hedra-compute event kinds the events route publishes.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxEventKind {
    Created,
    Draining,
    Stopping,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxEventKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Created => serializer.serialize_str("created"),
            Self::Draining => serializer.serialize_str("draining"),
            Self::Stopping => serializer.serialize_str("stopping"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxEventKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "created" => Ok(Self::Created),
            "draining" => Ok(Self::Draining),
            "stopping" => Ok(Self::Stopping),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Draining => write!(f, "draining"),
            Self::Stopping => write!(f, "stopping"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
