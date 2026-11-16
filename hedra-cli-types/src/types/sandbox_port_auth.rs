pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Who can call a sandbox's routed port.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxPortAuth {
    None,
    Token,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxPortAuth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::None => serializer.serialize_str("none"),
            Self::Token => serializer.serialize_str("token"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxPortAuth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "none" => Ok(Self::None),
            "token" => Ok(Self::Token),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxPortAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Token => write!(f, "token"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
