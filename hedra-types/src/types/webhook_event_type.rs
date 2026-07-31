pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The terminal events an outbound webhook announces.
/// 
/// One delivery carries exactly one of these, chosen from the job's final
/// status when the delivery fires.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WebhookEventType {
    JobCompleted,
    JobFailed,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for WebhookEventType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::JobCompleted => serializer.serialize_str("job.completed"),
            Self::JobFailed => serializer.serialize_str("job.failed"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for WebhookEventType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "job.completed" => Ok(Self::JobCompleted),
            "job.failed" => Ok(Self::JobFailed),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for WebhookEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JobCompleted => write!(f, "job.completed"),
            Self::JobFailed => write!(f, "job.failed"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
