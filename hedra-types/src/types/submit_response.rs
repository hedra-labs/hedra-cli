pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubmitResponse {
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub model: String,
    pub status: RequestStatus,
    #[serde(default)]
    pub status_url: String,
    #[serde(default)]
    pub response_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion_at: Option<String>,
}

impl SubmitResponse {
    pub fn builder() -> SubmitResponseBuilder {
        <SubmitResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitResponseBuilder {
    request_id: Option<String>,
    model: Option<String>,
    status: Option<RequestStatus>,
    status_url: Option<String>,
    response_url: Option<String>,
    estimated_completion_at: Option<String>,
}

impl SubmitResponseBuilder {
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

    pub fn status_url(mut self, value: impl Into<String>) -> Self {
        self.status_url = Some(value.into());
        self
    }

    pub fn response_url(mut self, value: impl Into<String>) -> Self {
        self.response_url = Some(value.into());
        self
    }

    pub fn estimated_completion_at(mut self, value: impl Into<String>) -> Self {
        self.estimated_completion_at = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`request_id`](SubmitResponseBuilder::request_id)
    /// - [`model`](SubmitResponseBuilder::model)
    /// - [`status`](SubmitResponseBuilder::status)
    /// - [`status_url`](SubmitResponseBuilder::status_url)
    /// - [`response_url`](SubmitResponseBuilder::response_url)
    pub fn build(self) -> Result<SubmitResponse, BuildError> {
        Ok(SubmitResponse {
            request_id: self.request_id.ok_or_else(|| BuildError::missing_field("request_id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            status_url: self.status_url.ok_or_else(|| BuildError::missing_field("status_url"))?,
            response_url: self.response_url.ok_or_else(|| BuildError::missing_field("response_url"))?,
            estimated_completion_at: self.estimated_completion_at,
        })
    }
}
