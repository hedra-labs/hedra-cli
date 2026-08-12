pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Where a delivery stands: queued, in flight, or its terminal outcome.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WebhookDeliveryStatus {
    Pending,
    Delivering,
    Delivered,
    Failed,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for WebhookDeliveryStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Pending => serializer.serialize_str("PENDING"),
            Self::Delivering => serializer.serialize_str("DELIVERING"),
            Self::Delivered => serializer.serialize_str("DELIVERED"),
            Self::Failed => serializer.serialize_str("FAILED"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for WebhookDeliveryStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "PENDING" => Ok(Self::Pending),
            "DELIVERING" => Ok(Self::Delivering),
            "DELIVERED" => Ok(Self::Delivered),
            "FAILED" => Ok(Self::Failed),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for WebhookDeliveryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Delivering => write!(f, "DELIVERING"),
            Self::Delivered => write!(f, "DELIVERED"),
            Self::Failed => write!(f, "FAILED"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
