pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Lifecycle of one chat-completion metering record. Same string values as
/// the stored rows (``db_models.llm_request.LlmRequestStatus``), re-declared
/// so the public OpenAPI publishes a surface-named component.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LlmUsageStatus {
    Pending,
    Ok,
    UpstreamError,
    ClientAbort,
    Timeout,
    SettlementFailed,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for LlmUsageStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Pending => serializer.serialize_str("pending"),
            Self::Ok => serializer.serialize_str("ok"),
            Self::UpstreamError => serializer.serialize_str("upstream_error"),
            Self::ClientAbort => serializer.serialize_str("client_abort"),
            Self::Timeout => serializer.serialize_str("timeout"),
            Self::SettlementFailed => serializer.serialize_str("settlement_failed"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for LlmUsageStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "pending" => Ok(Self::Pending),
            "ok" => Ok(Self::Ok),
            "upstream_error" => Ok(Self::UpstreamError),
            "client_abort" => Ok(Self::ClientAbort),
            "timeout" => Ok(Self::Timeout),
            "settlement_failed" => Ok(Self::SettlementFailed),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for LlmUsageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Ok => write!(f, "ok"),
            Self::UpstreamError => write!(f, "upstream_error"),
            Self::ClientAbort => write!(f, "client_abort"),
            Self::Timeout => write!(f, "timeout"),
            Self::SettlementFailed => write!(f, "settlement_failed"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
