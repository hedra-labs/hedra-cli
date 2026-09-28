pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The OpenAI error type for an error's HTTP status, published so an
/// OpenAI SDK's `APIStatusError.type` carries a value.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    InvalidRequestError,
    AuthenticationError,
    PermissionError,
    RateLimitError,
    ServerError,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for ErrorType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::InvalidRequestError => serializer.serialize_str("invalid_request_error"),
            Self::AuthenticationError => serializer.serialize_str("authentication_error"),
            Self::PermissionError => serializer.serialize_str("permission_error"),
            Self::RateLimitError => serializer.serialize_str("rate_limit_error"),
            Self::ServerError => serializer.serialize_str("server_error"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for ErrorType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "invalid_request_error" => Ok(Self::InvalidRequestError),
            "authentication_error" => Ok(Self::AuthenticationError),
            "permission_error" => Ok(Self::PermissionError),
            "rate_limit_error" => Ok(Self::RateLimitError),
            "server_error" => Ok(Self::ServerError),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequestError => write!(f, "invalid_request_error"),
            Self::AuthenticationError => write!(f, "authentication_error"),
            Self::PermissionError => write!(f, "permission_error"),
            Self::RateLimitError => write!(f, "rate_limit_error"),
            Self::ServerError => write!(f, "server_error"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
