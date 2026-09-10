pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `auto` — automatically selects rendering quality for the prompt. `low` — low rendering quality for the Sunburst model. `medium` — medium rendering quality for the Sunburst model. `high` — high rendering quality for the Sunburst model.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputGptImage25SunburstQuality {
    Auto,
    Low,
    Medium,
    High,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputGptImage25SunburstQuality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Low => serializer.serialize_str("low"),
            Self::Medium => serializer.serialize_str("medium"),
            Self::High => serializer.serialize_str("high"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputGptImage25SunburstQuality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "auto" => Ok(Self::Auto),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputGptImage25SunburstQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
