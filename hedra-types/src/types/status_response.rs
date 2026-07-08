pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusResponse {
    #[serde(default)]
    pub request_id: String,
    pub status: RequestStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<StatusLog>>,
}

impl StatusResponse {
    pub fn builder() -> StatusResponseBuilder {
        <StatusResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct StatusResponseBuilder {
    request_id: Option<String>,
    status: Option<RequestStatus>,
    progress: Option<f64>,
    estimated_completion_at: Option<String>,
    logs: Option<Vec<StatusLog>>,
}

impl StatusResponseBuilder {
    pub fn request_id(mut self, value: impl Into<String>) -> Self {
        self.request_id = Some(value.into());
        self
    }

    pub fn status(mut self, value: RequestStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn progress(mut self, value: f64) -> Self {
        self.progress = Some(value);
        self
    }

    pub fn estimated_completion_at(mut self, value: impl Into<String>) -> Self {
        self.estimated_completion_at = Some(value.into());
        self
    }

    pub fn logs(mut self, value: Vec<StatusLog>) -> Self {
        self.logs = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`StatusResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`request_id`](StatusResponseBuilder::request_id)
    /// - [`status`](StatusResponseBuilder::status)
    pub fn build(self) -> Result<StatusResponse, BuildError> {
        Ok(StatusResponse {
            request_id: self.request_id.ok_or_else(|| BuildError::missing_field("request_id"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            progress: self.progress,
            estimated_completion_at: self.estimated_completion_at,
            logs: self.logs,
        })
    }
}
