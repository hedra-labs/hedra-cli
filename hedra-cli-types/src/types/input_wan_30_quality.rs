pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `standard` — the base tier. `prime` — Wan's higher-fidelity tier over the same options, for final output.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputWan30Quality {
    Standard,
    Prime,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputWan30Quality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Standard => serializer.serialize_str("standard"),
            Self::Prime => serializer.serialize_str("prime"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputWan30Quality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "standard" => Ok(Self::Standard),
            "prime" => Ok(Self::Prime),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputWan30Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Prime => write!(f, "prime"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
