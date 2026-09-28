pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    /// Human-readable summary of the error. Fixed per condition — match on `code`, not on this text.
    #[serde(default)]
    pub message: String,
    /// Whether retrying the same request can succeed. Describes the condition, not a promise — pair with `retry_after` when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    /// Seconds to wait before retrying; set when the error is retryable. Mirrors the `Retry-After` response header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<i64>,
    /// The primary offending input field, when the error is about one specific field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    /// Every field-level problem, when the error is a validation failure — all of them at once, not just the first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
    /// The id of a successor model, set when the requested model has been retired (code `GONE`) and replaced; null otherwise. Lets a client programmatically migrate off a retired model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
    /// Balance, price, and where to add funds — set when the request was refused for funds (code `INSUFFICIENT_BALANCE`); null otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<BillingError>,
    /// The OpenAI error type for this error's HTTP status: `authentication_error` (401), `permission_error` (403), `rate_limit_error` (429), `server_error` (5xx), and `invalid_request_error` for every other status. Match on `code`, which is more specific.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ErrorType>,
}

impl ErrorEnvelope {
    pub fn builder() -> ErrorEnvelopeBuilder {
        <ErrorEnvelopeBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ErrorEnvelopeBuilder {
    code: Option<ErrorCode>,
    message: Option<String>,
    retryable: Option<bool>,
    retry_after: Option<i64>,
    param: Option<String>,
    details: Option<Vec<FieldError>>,
    replaced_by: Option<String>,
    billing: Option<BillingError>,
    r#type: Option<ErrorType>,
}

impl ErrorEnvelopeBuilder {
    pub fn code(mut self, value: ErrorCode) -> Self {
        self.code = Some(value);
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn retryable(mut self, value: bool) -> Self {
        self.retryable = Some(value);
        self
    }

    pub fn retry_after(mut self, value: i64) -> Self {
        self.retry_after = Some(value);
        self
    }

    pub fn param(mut self, value: impl Into<String>) -> Self {
        self.param = Some(value.into());
        self
    }

    pub fn details(mut self, value: Vec<FieldError>) -> Self {
        self.details = Some(value);
        self
    }

    pub fn replaced_by(mut self, value: impl Into<String>) -> Self {
        self.replaced_by = Some(value.into());
        self
    }

    pub fn billing(mut self, value: BillingError) -> Self {
        self.billing = Some(value);
        self
    }

    pub fn r#type(mut self, value: ErrorType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ErrorEnvelope`].
    /// This method will fail if any of the following fields are not set:
    /// - [`code`](ErrorEnvelopeBuilder::code)
    /// - [`message`](ErrorEnvelopeBuilder::message)
    pub fn build(self) -> Result<ErrorEnvelope, BuildError> {
        Ok(ErrorEnvelope {
            code: self.code.ok_or_else(|| BuildError::missing_field("code"))?,
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
            retryable: self.retryable,
            retry_after: self.retry_after,
            param: self.param,
            details: self.details,
            replaced_by: self.replaced_by,
            billing: self.billing,
            r#type: self.r#type,
        })
    }
}
