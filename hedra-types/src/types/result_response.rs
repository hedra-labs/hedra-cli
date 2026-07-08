pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ResultResponse {
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub model: String,
    pub status: RequestStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<OutputItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Metrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<StatusLog>>,
}

impl ResultResponse {
    pub fn builder() -> ResultResponseBuilder {
        <ResultResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ResultResponseBuilder {
    request_id: Option<String>,
    model: Option<String>,
    status: Option<RequestStatus>,
    outputs: Option<Vec<OutputItem>>,
    metrics: Option<Metrics>,
    error: Option<ErrorEnvelope>,
    logs: Option<Vec<StatusLog>>,
}

impl ResultResponseBuilder {
    pub fn request_id(mut self, value: impl Into<String>) -> Self {
        self.request_id = Some(value.into());
        self
    }

    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn status(mut self, value: RequestStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn outputs(mut self, value: Vec<OutputItem>) -> Self {
        self.outputs = Some(value);
        self
    }

    pub fn metrics(mut self, value: Metrics) -> Self {
        self.metrics = Some(value);
        self
    }

    pub fn error(mut self, value: ErrorEnvelope) -> Self {
        self.error = Some(value);
        self
    }

    pub fn logs(mut self, value: Vec<StatusLog>) -> Self {
        self.logs = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ResultResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`request_id`](ResultResponseBuilder::request_id)
    /// - [`model`](ResultResponseBuilder::model)
    /// - [`status`](ResultResponseBuilder::status)
    pub fn build(self) -> Result<ResultResponse, BuildError> {
        Ok(ResultResponse {
            request_id: self.request_id.ok_or_else(|| BuildError::missing_field("request_id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            outputs: self.outputs,
            metrics: self.metrics,
            error: self.error,
            logs: self.logs,
        })
    }
}
