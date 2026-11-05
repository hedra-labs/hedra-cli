pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `pro` — the base tier. `max` — the higher-fidelity tier, for complex multi-element scenes, at twice the rate.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputFluxKontextQuality {
    Pro,
    Max,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputFluxKontextQuality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Pro => serializer.serialize_str("pro"),
            Self::Max => serializer.serialize_str("max"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputFluxKontextQuality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "pro" => Ok(Self::Pro),
            "max" => Ok(Self::Max),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputFluxKontextQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pro => write!(f, "pro"),
            Self::Max => write!(f, "max"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
