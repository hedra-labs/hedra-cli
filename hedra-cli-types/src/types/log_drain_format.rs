pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Wire encoding for one log drain endpoint's HTTP posts.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogDrainFormat {
    Ndjson,
    Otlp,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for LogDrainFormat {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Ndjson => serializer.serialize_str("ndjson"),
            Self::Otlp => serializer.serialize_str("otlp"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for LogDrainFormat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "ndjson" => Ok(Self::Ndjson),
            "otlp" => Ok(Self::Otlp),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for LogDrainFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ndjson => write!(f, "ndjson"),
            Self::Otlp => write!(f, "otlp"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
