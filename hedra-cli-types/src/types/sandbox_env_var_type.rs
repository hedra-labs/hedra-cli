pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A workspace environment variable's type, chosen when it is set and
/// fixed after: a ``config`` value any member can read back, or a
/// ``secret`` value that is never returned.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxEnvVarType {
    Config,
    Secret,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxEnvVarType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Config => serializer.serialize_str("config"),
            Self::Secret => serializer.serialize_str("secret"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxEnvVarType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "config" => Ok(Self::Config),
            "secret" => Ok(Self::Secret),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxEnvVarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config => write!(f, "config"),
            Self::Secret => write!(f, "secret"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
