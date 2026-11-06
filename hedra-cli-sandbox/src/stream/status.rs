//! A broker gRPC status, with the reason its `ErrorInfo` names.

use tonic::Code;
use tonic_types::StatusExt;

/// The `ErrorInfo.domain` the broker's reasons are in.
pub const DOMAIN: &str = "sandbox.hedra.dev";

/// A status the broker or the transport ended a call with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerStatus {
    pub code: Code,
    /// The broker's reason, such as `permission_denied`, when the status
    /// carries an `ErrorInfo` in the broker's domain.
    pub reason: Option<String>,
    pub message: String,
    /// The `ErrorInfo` metadata. The broker names a `permission_denied`'s
    /// scope only in `message`, not here.
    pub metadata: Vec<(String, String)>,
}

impl BrokerStatus {
    pub fn is(&self, reason: &str) -> bool {
        self.reason.as_deref() == Some(reason)
    }
}

/// The status's code, reason, message and metadata, for `--debug`.
impl std::fmt::Display for BrokerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "code {:?}, reason {}, message {:?}, metadata {:?}",
            self.code,
            self.reason.as_deref().unwrap_or("none"),
            self.message,
            self.metadata
        )
    }
}

impl From<&tonic::Status> for BrokerStatus {
    fn from(status: &tonic::Status) -> Self {
        let info = status
            .get_details_error_info()
            .filter(|info| info.domain == DOMAIN);
        let mut metadata: Vec<(String, String)> = info
            .as_ref()
            .map(|i| {
                i.metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();
        metadata.sort();
        Self {
            code: status.code(),
            reason: info.map(|i| i.reason),
            message: status.message().to_string(),
            metadata,
        }
    }
}

/// The reasons the broker sends that the client acts on.
pub mod reason {
    pub const PERMISSION_DENIED: &str = "permission_denied";
    pub const QUOTA_EXCEEDED: &str = "quota_exceeded";
    pub const SANDBOX_DISCONNECTED: &str = "sandbox_disconnected";
    pub const RATE_LIMITED: &str = "rate_limited";
    pub const NOT_FOUND: &str = "not_found";
    pub const INTERNAL: &str = "internal";
    pub const INVALID_ARGUMENT: &str = "invalid_argument";
    pub const UNAUTHENTICATED: &str = "unauthenticated";
}

/// Whether a refusal before admission is worth retrying: the supervisor is
/// not connected yet, or the broker could not be reached or gave a reason
/// this client does not know.
pub fn retry_before_admission(s: &BrokerStatus) -> bool {
    if s.is(reason::SANDBOX_DISCONNECTED) {
        return true;
    }
    let known = s.reason.as_deref().is_some_and(is_known);
    matches!(s.code, Code::Unavailable | Code::DeadlineExceeded) && !known
}

fn is_known(r: &str) -> bool {
    use reason::*;
    [
        PERMISSION_DENIED,
        QUOTA_EXCEEDED,
        SANDBOX_DISCONNECTED,
        RATE_LIMITED,
        NOT_FOUND,
        INTERNAL,
        INVALID_ARGUMENT,
        UNAUTHENTICATED,
    ]
    .contains(&r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic_types::{ErrorDetails, StatusExt};

    fn status(code: Code, domain: &str, reason: &str) -> tonic::Status {
        let details = ErrorDetails::with_error_info(
            reason,
            domain,
            [("field".to_string(), "replica".to_string())],
        );
        tonic::Status::with_error_details(code, "msg", details)
    }

    #[test]
    fn the_reason_is_read_only_from_the_brokers_domain() {
        let s = BrokerStatus::from(&status(Code::PermissionDenied, DOMAIN, "permission_denied"));
        assert!(s.is(reason::PERMISSION_DENIED));
        assert_eq!(s.metadata, [("field".to_string(), "replica".to_string())]);
        let other = BrokerStatus::from(&status(
            Code::PermissionDenied,
            "example.com",
            "permission_denied",
        ));
        assert_eq!(other.reason, None);
        let bare = BrokerStatus::from(&tonic::Status::unavailable("gone"));
        assert_eq!(bare.reason, None);
    }

    #[test]
    fn retries_before_admission_cover_disconnected_and_unexplained_unavailable() {
        let r = |c, d, why| retry_before_admission(&BrokerStatus::from(&status(c, d, why)));
        assert!(r(Code::Unavailable, DOMAIN, "sandbox_disconnected"));
        assert!(r(Code::Unavailable, DOMAIN, "a_new_reason"));
        assert!(r(Code::DeadlineExceeded, "x", "y"));
        assert!(!r(Code::Unavailable, DOMAIN, "rate_limited"));
        assert!(!r(Code::PermissionDenied, DOMAIN, "permission_denied"));
        assert!(retry_before_admission(&BrokerStatus::from(
            &tonic::Status::unavailable("x")
        )));
        assert!(!retry_before_admission(&BrokerStatus::from(
            &tonic::Status::internal("x")
        )));
    }
}
