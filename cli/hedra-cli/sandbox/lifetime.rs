//! Reservation limits shared by generated creates and GPU shells.

use fern_cli_sdk::error::CliError;
use fern_cli_sdk::openapi::discovery::RestMethod;
use serde_json::Value;

pub(crate) const MIN_SECONDS: i64 = 120;
pub(crate) const MAX_SECONDS: i64 = 86_400;
const LIMIT_MESSAGE: &str = "Sandbox reservations require ttl_seconds between 120 and 86400 (2 minutes to 24 hours). Longer reservations require a sales contract with Hedra; contact sales@hedra.com.";

pub(crate) fn check(seconds: i64) -> Result<i64, String> {
    if (MIN_SECONDS..=MAX_SECONDS).contains(&seconds) {
        Ok(seconds)
    } else {
        Err(LIMIT_MESSAGE.into())
    }
}

pub(crate) fn validate_body(method: &RestMethod, body: Option<&Value>) -> Result<(), CliError> {
    if method.http_method.eq_ignore_ascii_case("POST")
        && method.path.trim_matches('/') == "sandboxes"
    {
        let seconds = body
            .and_then(|body| body.get("ttl_seconds"))
            .and_then(Value::as_i64)
            .ok_or_else(|| CliError::Validation(LIMIT_MESSAGE.into()))?;
        check(seconds).map_err(CliError::Validation)?;
    }
    Ok(())
}

pub(crate) fn install() {
    fern_cli_sdk::openapi::executor::set_body_validator(validate_body);
}
