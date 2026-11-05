pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `pro` — the everyday tier, for style transfer and sequential edits. `max` — the top tier, for realism and precision in final deliverables, at the highest price before reference images. `flex` — the tunable tier, adding `guidance` and `steps` to trade fidelity against speed.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputFlux2Quality {
    Pro,
    Max,
    Flex,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputFlux2Quality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Pro => serializer.serialize_str("pro"),
            Self::Max => serializer.serialize_str("max"),
            Self::Flex => serializer.serialize_str("flex"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputFlux2Quality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "pro" => Ok(Self::Pro),
            "max" => Ok(Self::Max),
            "flex" => Ok(Self::Flex),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputFlux2Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pro => write!(f, "pro"),
            Self::Max => write!(f, "max"),
            Self::Flex => write!(f, "flex"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
