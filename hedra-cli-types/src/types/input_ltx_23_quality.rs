pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `fast` — tuned for turnaround, for iteration. `pro` — the higher-fidelity tier, for final output.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputLtx23Quality {
    Fast,
    Pro,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputLtx23Quality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Fast => serializer.serialize_str("fast"),
            Self::Pro => serializer.serialize_str("pro"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputLtx23Quality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "fast" => Ok(Self::Fast),
            "pro" => Ok(Self::Pro),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputLtx23Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fast => write!(f, "fast"),
            Self::Pro => write!(f, "pro"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
