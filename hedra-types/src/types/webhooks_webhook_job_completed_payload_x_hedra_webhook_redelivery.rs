pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WebhookJobCompletedPayloadXHedraWebhookRedelivery {
    True,
    False,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for WebhookJobCompletedPayloadXHedraWebhookRedelivery {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::True => serializer.serialize_str("true"),
            Self::False => serializer.serialize_str("false"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for WebhookJobCompletedPayloadXHedraWebhookRedelivery {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for WebhookJobCompletedPayloadXHedraWebhookRedelivery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
