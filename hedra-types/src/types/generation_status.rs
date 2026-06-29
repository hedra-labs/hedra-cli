pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenerationStatus {
    Complete,
    Error,
    Processing,
    Queued,
    Finalizing,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for GenerationStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Complete => serializer.serialize_str("complete"),
            Self::Error => serializer.serialize_str("error"),
            Self::Processing => serializer.serialize_str("processing"),
            Self::Queued => serializer.serialize_str("queued"),
            Self::Finalizing => serializer.serialize_str("finalizing"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for GenerationStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "complete" => Ok(Self::Complete),
            "error" => Ok(Self::Error),
            "processing" => Ok(Self::Processing),
            "queued" => Ok(Self::Queued),
            "finalizing" => Ok(Self::Finalizing),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for GenerationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "complete"),
            Self::Error => write!(f, "error"),
            Self::Processing => write!(f, "processing"),
            Self::Queued => write!(f, "queued"),
            Self::Finalizing => write!(f, "finalizing"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
