pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// How the sandbox is paid for, fixed when it is created.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxBillingMode {
    Lease,
    CapacityReservation,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxBillingMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Lease => serializer.serialize_str("lease"),
            Self::CapacityReservation => serializer.serialize_str("capacity_reservation"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxBillingMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "lease" => Ok(Self::Lease),
            "capacity_reservation" => Ok(Self::CapacityReservation),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxBillingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lease => write!(f, "lease"),
            Self::CapacityReservation => write!(f, "capacity_reservation"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}
