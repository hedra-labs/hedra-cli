pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobLogSource {
    Api,
    Worker,
    Provider,
    Cron,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for JobLogSource {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Api => serializer.serialize_str("api"),
            Self::Worker => serializer.serialize_str("worker"),
            Self::Provider => serializer.serialize_str("provider"),
            Self::Cron => serializer.serialize_str("cron"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for JobLogSource {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "api" => Ok(Self::Api),
            "worker" => Ok(Self::Worker),
            "provider" => Ok(Self::Provider),
            "cron" => Ok(Self::Cron),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for JobLogSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api => write!(f, "api"),
            Self::Worker => write!(f, "worker"),
            Self::Provider => write!(f, "provider"),
            Self::Cron => write!(f, "cron"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
