pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Personal keys die with the member; service keys belong to the workspace.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApiKeyKind {
    Personal,
    Service,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for ApiKeyKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Personal => serializer.serialize_str("personal"),
            Self::Service => serializer.serialize_str("service"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for ApiKeyKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "personal" => Ok(Self::Personal),
            "service" => Ok(Self::Service),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for ApiKeyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Personal => write!(f, "personal"),
            Self::Service => write!(f, "service"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
