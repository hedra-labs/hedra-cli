pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// When hedra-compute restarts the command; hedra-compute's own values.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxRestartPolicy {
    Never,
    OnFailure,
    Always,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxRestartPolicy {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Never => serializer.serialize_str("never"),
            Self::OnFailure => serializer.serialize_str("on-failure"),
            Self::Always => serializer.serialize_str("always"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxRestartPolicy {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "never" => Ok(Self::Never),
            "on-failure" => Ok(Self::OnFailure),
            "always" => Ok(Self::Always),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxRestartPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never => write!(f, "never"),
            Self::OnFailure => write!(f, "on-failure"),
            Self::Always => write!(f, "always"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
