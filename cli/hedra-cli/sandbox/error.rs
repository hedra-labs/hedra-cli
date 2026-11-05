//! The `sandbox` group's failure classes, exit codes and error output.
//!
//! The runtime maps every HTTP answer to exit code 1, so the group writes its
//! own errors and exits with one code per class. It keeps the runtime's 1, 2
//! and 3 and starts its own classes at 10, so no code means two things.

use serde_json::{json, Value};

/// The v3 API messages the verbs tell 404s apart by. They are plain
/// `NotFoundError`s with the same `code`, so the message is the only signal.
pub(crate) const SANDBOX_NOT_FOUND: &str = "Sandbox not found.";
pub(crate) const SANDBOXES_NOT_AVAILABLE: &str = "Sandboxes are not available.";
pub(crate) const SECRET_NOT_FOUND: &str = "Secret not found.";

const SIGN_IN: &str = "Sign in with `hedra-cli auth login`, or set HEDRA_API_KEY.";
const TEMPORARILY_UNAVAILABLE: &str = "Hedra is temporarily unavailable; retry shortly.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Class {
    Internal,
    NotSignedIn,
    Interrupted,
    MissingScope,
    NotFound,
    NotEnabled,
    NotReady,
    NoCapacity,
    Payment,
    Limit,
    Unavailable,
    ConnectionLost,
    Rejected,
    OutputFailed,
    Hangup,
    Terminated,
}

impl Class {
    pub(crate) fn code(self) -> i32 {
        match self {
            Class::Internal => 1,
            Class::NotSignedIn => 2,
            Class::MissingScope => 10,
            Class::NotFound => 11,
            Class::NotEnabled => 12,
            Class::NotReady => 13,
            Class::NoCapacity => 14,
            Class::Payment => 15,
            Class::Limit => 16,
            Class::Unavailable => 17,
            Class::ConnectionLost => 18,
            Class::Rejected => 19,
            Class::OutputFailed => 255,
            Class::Hangup => 129,
            Class::Terminated => 143,
            Class::Interrupted => 130,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Class::Internal => "Internal",
            Class::NotSignedIn => "Not signed in",
            Class::Interrupted => "Interrupted",
            Class::MissingScope => "Missing scope",
            Class::NotFound => "Not found",
            Class::NotEnabled => "Not enabled",
            Class::NotReady => "Not ready or stopped",
            Class::NoCapacity => "No capacity",
            Class::Payment => "Payment",
            Class::Limit => "Limit",
            Class::Unavailable => "Unavailable",
            Class::ConnectionLost => "Connection lost",
            Class::Rejected => "Rejected request",
            Class::OutputFailed => "Output failed",
            Class::Hangup => "Hangup",
            Class::Terminated => "Terminated",
        }
    }
}

/// One failure: its class, the line a person reads, and what the server said.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Failure {
    pub(crate) class: Class,
    pub(crate) message: String,
    pub(crate) reason: Option<String>,
    pub(crate) trace_id: Option<String>,
}

impl Failure {
    pub(crate) fn new(class: Class, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
            reason: None,
            trace_id: None,
        }
    }

    pub(crate) fn rejected(message: impl Into<String>) -> Self {
        Self::new(Class::Rejected, message)
    }

    pub(crate) fn not_found(subject: &str) -> Self {
        Self::new(
            Class::NotFound,
            format!("No sandbox `{subject}`. Run `hedra-cli sandbox ls`."),
        )
    }

    pub(crate) fn transport(detail: &str) -> Self {
        Self::new(
            Class::Unavailable,
            format!("{TEMPORARILY_UNAVAILABLE} ({detail})"),
        )
    }

    pub(crate) fn with_trace(mut self, trace_id: Option<String>) -> Self {
        self.trace_id = trace_id;
        self
    }

    /// Appends a sentence to the message, as the next step.
    pub(crate) fn then(mut self, next: impl AsRef<str>) -> Self {
        self.message.push(' ');
        self.message.push_str(next.as_ref());
        self
    }

    /// `hedra-cli: <class> (<code>): <message>`, control characters escaped.
    pub(crate) fn line(&self) -> String {
        format!(
            "hedra-cli: {} ({}): {}",
            self.class.name(),
            self.class.code(),
            escape_controls(&self.message)
        )
    }

    pub(crate) fn json(&self) -> Value {
        json!({
            "error": {
                "class": self.class.name(),
                "exit_code": self.class.code(),
                "message": self.message,
                "reason": self.reason,
                "trace_id": self.trace_id,
            }
        })
    }

    /// A verb whose stdout is the sandbox's output writes every error to
    /// stderr, as JSON only when a machine format was asked for explicitly.
    pub(crate) fn write_stderr(&self, json: bool) {
        if json {
            super::stream::note(self.json());
        } else {
            super::stream::note(self.line());
        }
    }
}

/// The v3 error envelope, read leniently: a load balancer's 502 or a
/// proxy's page has none.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Envelope {
    pub(crate) code: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) retry_after: Option<u64>,
    pub(crate) trace_id: Option<String>,
}

impl Envelope {
    pub(crate) fn parse(body: &str) -> Self {
        let Ok(v) = serde_json::from_str::<Value>(body) else {
            return Self::default();
        };
        let error = v.get("error").filter(|e| e.is_object());
        let str_at = |key: &str| {
            error
                .and_then(|e| e.get(key))
                .and_then(Value::as_str)
                .map(str::to_string)
        };
        Self {
            code: str_at("code"),
            message: str_at("message"),
            retry_after: error
                .and_then(|e| e.get("retry_after"))
                .and_then(Value::as_u64),
            trace_id: v
                .get("trace_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        }
    }

    fn has_code(&self, code: &str) -> bool {
        self.code.as_deref() == Some(code)
    }
}

/// An HTTP error answer from a v3 sandbox route.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HttpError {
    pub(crate) status: u16,
    pub(crate) envelope: Envelope,
}

impl HttpError {
    pub(crate) fn new(status: u16, body: &str) -> Self {
        Self {
            status,
            envelope: Envelope::parse(body),
        }
    }

    pub(crate) fn message(&self) -> &str {
        self.envelope.message.as_deref().unwrap_or("")
    }

    pub(crate) fn is(&self, status: u16, code: &str) -> bool {
        self.status == status && self.envelope.has_code(code)
    }

    /// The scope a 403 names: "Key is missing required scope(s): sandboxes:X."
    pub(crate) fn missing_scope(&self) -> Option<String> {
        if self.status != 403 {
            return None;
        }
        let (_, rest) = self.message().split_once("missing required scope(s):")?;
        let scope = rest.trim().trim_end_matches('.').trim();
        (!scope.is_empty()).then(|| scope.to_string())
    }

    /// The exit-code class and message for this answer. `subject` is the name
    /// or id the caller asked for, for the Not found line.
    pub(crate) fn classify(&self, subject: Option<&str>) -> Failure {
        let message = self.message().to_string();
        let (class, line) = match self.status {
            401 => (Class::NotSignedIn, SIGN_IN.to_string()),
            402 => (Class::Payment, message),
            403 => match self.missing_scope() {
                Some(scope) => (Class::MissingScope, missing_scope_line(&scope)),
                None => (Class::Rejected, message),
            },
            404 if message == SANDBOX_NOT_FOUND => match subject {
                Some(subject) => return self.with_server(Failure::not_found(subject)),
                None => (Class::NotFound, message),
            },
            404 if message == SANDBOXES_NOT_AVAILABLE => (
                Class::NotEnabled,
                "Sandboxes are not available to this workspace yet. Ask Hedra to enable them."
                    .to_string(),
            ),
            404 if message == SECRET_NOT_FOUND => (Class::Rejected, message),
            400 if self.envelope.has_code("FAILED_PRECONDITION") => (Class::NotReady, message),
            409 if self.envelope.has_code("ALREADY_IN_PROGRESS") => (Class::Unavailable, message),
            429 => (Class::Limit, message),
            500 => (
                Class::Internal,
                match &self.envelope.trace_id {
                    Some(trace) => format!(
                        "Hedra returned an error (trace {trace}). Retry, or contact support with the trace."
                    ),
                    None => "Hedra returned an error. Retry, or contact support.".to_string(),
                },
            ),
            503 if self.envelope.message.is_some() => (Class::Unavailable, message),
            502..=504 => (Class::Unavailable, TEMPORARILY_UNAVAILABLE.to_string()),
            s if s >= 500 => (
                Class::Internal,
                format!("Hedra returned an error (HTTP {s}). Retry, or contact support."),
            ),
            _ => (Class::Rejected, message),
        };
        let line = if line.is_empty() {
            format!("Hedra answered HTTP {}.", self.status)
        } else {
            line
        };
        self.with_server(Failure::new(class, line))
    }

    fn with_server(&self, mut failure: Failure) -> Failure {
        failure.reason = self.envelope.code.clone();
        failure.trace_id = self.envelope.trace_id.clone();
        failure
    }
}

/// The Missing scope line, which names the next step for the credential in
/// use: `hedra-cli auth login` cannot replace a key given in `HEDRA_API_KEY`.
pub(crate) fn missing_scope_line(scope: &str) -> String {
    let api_key_set = std::env::var_os("HEDRA_API_KEY").is_some_and(|v| !v.is_empty());
    missing_scope_line_for(scope, api_key_set)
}

fn missing_scope_line_for(scope: &str, api_key_set: bool) -> String {
    if api_key_set {
        return format!("This key lacks `{scope}`. Set HEDRA_API_KEY to a key with it.");
    }
    format!(
        "This key lacks `{scope}`. Run `hedra-cli auth login` to get a new key, or ask a \
         workspace owner or admin for one with it."
    )
}

/// Escapes control characters in text that came from a server, so it cannot
/// send a terminal escape sequence through a line hedra-cli prints.
pub(crate) fn escape_controls(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_control() {
            out.extend(c.escape_default());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(code: &str, message: &str) -> String {
        json!({
            "error": {"code": code, "message": message, "retryable": false},
            "trace_id": "trace_1",
        })
        .to_string()
    }

    fn class_of(status: u16, code: &str, message: &str) -> Class {
        HttpError::new(status, &envelope(code, message))
            .classify(Some("train"))
            .class
    }

    #[test]
    fn each_answer_maps_to_its_class() {
        assert_eq!(class_of(401, "UNAUTHENTICATED", "x"), Class::NotSignedIn);
        assert_eq!(class_of(402, "INSUFFICIENT_BALANCE", "x"), Class::Payment);
        assert_eq!(
            class_of(
                403,
                "PERMISSION_DENIED",
                "Key is missing required scope(s): sandboxes:write."
            ),
            Class::MissingScope
        );
        assert_eq!(
            class_of(
                403,
                "PERMISSION_DENIED",
                "Only a workspace owner or admin can do this."
            ),
            Class::Rejected
        );
        assert_eq!(
            class_of(404, "NOT_FOUND", SANDBOX_NOT_FOUND),
            Class::NotFound
        );
        assert_eq!(
            class_of(404, "NOT_FOUND", SANDBOXES_NOT_AVAILABLE),
            Class::NotEnabled
        );
        assert_eq!(
            class_of(404, "NOT_FOUND", SECRET_NOT_FOUND),
            Class::Rejected
        );
        assert_eq!(
            class_of(404, "NOT_FOUND", "Account not found."),
            Class::Rejected
        );
        assert_eq!(class_of(400, "FAILED_PRECONDITION", "x"), Class::NotReady);
        assert_eq!(class_of(400, "INVALID_ARGUMENT", "x"), Class::Rejected);
        assert_eq!(class_of(409, "ALREADY_EXISTS", "x"), Class::Rejected);
        assert_eq!(
            class_of(409, "ALREADY_IN_PROGRESS", "x"),
            Class::Unavailable
        );
        assert_eq!(class_of(429, "RATE_LIMITED", "x"), Class::Limit);
        assert_eq!(class_of(500, "INTERNAL", "x"), Class::Internal);
        assert_eq!(class_of(503, "UNAVAILABLE", "x"), Class::Unavailable);
    }

    #[test]
    fn a_gateway_answer_without_an_envelope_is_unavailable_with_no_trace() {
        for status in [502, 503, 504] {
            let f = HttpError::new(status, "<html>Bad gateway</html>").classify(None);
            assert_eq!(f.class, Class::Unavailable);
            assert_eq!(f.message, TEMPORARILY_UNAVAILABLE);
            assert_eq!(f.trace_id, None);
        }
    }

    #[test]
    fn a_missing_scope_names_the_scope() {
        let f = HttpError::new(
            403,
            &envelope(
                "PERMISSION_DENIED",
                "Key is missing required scope(s): sandboxes:write.",
            ),
        )
        .classify(None);
        assert!(f.message.contains("`sandboxes:write`"), "{}", f.message);
    }

    #[test]
    fn not_found_names_the_subject_and_keeps_the_trace() {
        let f =
            HttpError::new(404, &envelope("NOT_FOUND", SANDBOX_NOT_FOUND)).classify(Some("train"));
        assert_eq!(f.message, "No sandbox `train`. Run `hedra-cli sandbox ls`.");
        assert_eq!(f.trace_id.as_deref(), Some("trace_1"));
        assert_eq!(f.reason.as_deref(), Some("NOT_FOUND"));
    }

    #[test]
    fn retry_after_is_read_from_the_envelope() {
        let body = json!({"error": {"code": "RATE_LIMITED", "message": "x", "retry_after": 7}})
            .to_string();
        assert_eq!(Envelope::parse(&body).retry_after, Some(7));
    }

    #[test]
    fn the_line_has_the_class_prefix_and_escapes_controls() {
        let f = Failure::rejected("bad \x1b]52;c;aGk=\x07 name");
        assert_eq!(
            f.line(),
            "hedra-cli: Rejected request (19): bad \\u{1b}]52;c;aGk=\\u{7} name"
        );
    }

    #[test]
    fn the_json_carries_class_code_reason_and_trace() {
        let f = HttpError::new(429, &envelope("RATE_LIMITED", "Slow down.")).classify(None);
        assert_eq!(
            f.json(),
            json!({"error": {
                "class": "Limit", "exit_code": 16, "message": "Slow down.",
                "reason": "RATE_LIMITED", "trace_id": "trace_1",
            }})
        );
    }

    #[test]
    fn the_missing_scope_line_says_to_sign_in_again() {
        assert_eq!(
            missing_scope_line_for("sandboxes:exec", false),
            "This key lacks `sandboxes:exec`. Run `hedra-cli auth login` to get a new key, \
             or ask a workspace owner or admin for one with it."
        );
    }

    #[test]
    fn with_hedra_api_key_set_the_missing_scope_line_says_to_replace_it() {
        assert_eq!(
            missing_scope_line_for("sandboxes:read", true),
            "This key lacks `sandboxes:read`. Set HEDRA_API_KEY to a key with it."
        );
    }
}
