pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct LogDrainTestResponse {
    #[serde(default)]
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_status: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl LogDrainTestResponse {
    pub fn builder() -> LogDrainTestResponseBuilder {
        <LogDrainTestResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LogDrainTestResponseBuilder {
    ok: Option<bool>,
    response_status: Option<i64>,
    error: Option<String>,
}

impl LogDrainTestResponseBuilder {
    pub fn ok(mut self, value: bool) -> Self {
        self.ok = Some(value);
        self
    }

    pub fn response_status(mut self, value: i64) -> Self {
        self.response_status = Some(value);
        self
    }

    pub fn error(mut self, value: impl Into<String>) -> Self {
        self.error = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`LogDrainTestResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`ok`](LogDrainTestResponseBuilder::ok)
    pub fn build(self) -> Result<LogDrainTestResponse, BuildError> {
        Ok(LogDrainTestResponse {
            ok: self.ok.ok_or_else(|| BuildError::missing_field("ok"))?,
            response_status: self.response_status,
            error: self.error,
        })
    }
}
