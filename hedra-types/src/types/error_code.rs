pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Universal error code space. Modeled after gRPC status codes.
/// 
/// Every exception in the system carries an ErrorCode. SDK clients map
/// 3rd-party errors to these codes at the lowest level. Retry logic,
/// HTTP status mapping, and OTEL metrics all key off this enum.
/// 
/// NOTE: The semantic meaning of these error codes is roughly mapped from the
/// semantic of the gRPC status codes: see
/// https://grpc.io/docs/guides/status-codes/ for details.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    Unknown,
    InvalidArgument,
    NotFound,
    AlreadyExists,
    AlreadyInProgress,
    Unauthorized,
    PermissionDenied,
    MissingCredits,
    ModerationFailed,
    FailedPrecondition,
    DeadlineExceeded,
    ResourceExhausted,
    Unavailable,
    Internal,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for ErrorCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Unknown => serializer.serialize_str("UNKNOWN"),
            Self::InvalidArgument => serializer.serialize_str("INVALID_ARGUMENT"),
            Self::NotFound => serializer.serialize_str("NOT_FOUND"),
            Self::AlreadyExists => serializer.serialize_str("ALREADY_EXISTS"),
            Self::AlreadyInProgress => serializer.serialize_str("ALREADY_IN_PROGRESS"),
            Self::Unauthorized => serializer.serialize_str("UNAUTHORIZED"),
            Self::PermissionDenied => serializer.serialize_str("PERMISSION_DENIED"),
            Self::MissingCredits => serializer.serialize_str("MISSING_CREDITS"),
            Self::ModerationFailed => serializer.serialize_str("MODERATION_FAILED"),
            Self::FailedPrecondition => serializer.serialize_str("FAILED_PRECONDITION"),
            Self::DeadlineExceeded => serializer.serialize_str("DEADLINE_EXCEEDED"),
            Self::ResourceExhausted => serializer.serialize_str("RESOURCE_EXHAUSTED"),
            Self::Unavailable => serializer.serialize_str("UNAVAILABLE"),
            Self::Internal => serializer.serialize_str("INTERNAL"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "UNKNOWN" => Ok(Self::Unknown),
            "INVALID_ARGUMENT" => Ok(Self::InvalidArgument),
            "NOT_FOUND" => Ok(Self::NotFound),
            "ALREADY_EXISTS" => Ok(Self::AlreadyExists),
            "ALREADY_IN_PROGRESS" => Ok(Self::AlreadyInProgress),
            "UNAUTHORIZED" => Ok(Self::Unauthorized),
            "PERMISSION_DENIED" => Ok(Self::PermissionDenied),
            "MISSING_CREDITS" => Ok(Self::MissingCredits),
            "MODERATION_FAILED" => Ok(Self::ModerationFailed),
            "FAILED_PRECONDITION" => Ok(Self::FailedPrecondition),
            "DEADLINE_EXCEEDED" => Ok(Self::DeadlineExceeded),
            "RESOURCE_EXHAUSTED" => Ok(Self::ResourceExhausted),
            "UNAVAILABLE" => Ok(Self::Unavailable),
            "INTERNAL" => Ok(Self::Internal),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::InvalidArgument => write!(f, "INVALID_ARGUMENT"),
            Self::NotFound => write!(f, "NOT_FOUND"),
            Self::AlreadyExists => write!(f, "ALREADY_EXISTS"),
            Self::AlreadyInProgress => write!(f, "ALREADY_IN_PROGRESS"),
            Self::Unauthorized => write!(f, "UNAUTHORIZED"),
            Self::PermissionDenied => write!(f, "PERMISSION_DENIED"),
            Self::MissingCredits => write!(f, "MISSING_CREDITS"),
            Self::ModerationFailed => write!(f, "MODERATION_FAILED"),
            Self::FailedPrecondition => write!(f, "FAILED_PRECONDITION"),
            Self::DeadlineExceeded => write!(f, "DEADLINE_EXCEEDED"),
            Self::ResourceExhausted => write!(f, "RESOURCE_EXHAUSTED"),
            Self::Unavailable => write!(f, "UNAVAILABLE"),
            Self::Internal => write!(f, "INTERNAL"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
