pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Which URL resolution created this delivery.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WebhookDeliverySource {
    PerJob,
    Default,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for WebhookDeliverySource {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::PerJob => serializer.serialize_str("per_job"),
            Self::Default => serializer.serialize_str("default"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for WebhookDeliverySource {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "per_job" => Ok(Self::PerJob),
            "default" => Ok(Self::Default),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for WebhookDeliverySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PerJob => write!(f, "per_job"),
            Self::Default => write!(f, "default"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
