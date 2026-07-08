pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RequestSummary {
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub model: String,
    pub status: RequestStatus,
}

impl RequestSummary {
    pub fn builder() -> RequestSummaryBuilder {
        <RequestSummaryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct RequestSummaryBuilder {
    request_id: Option<String>,
    model: Option<String>,
    status: Option<RequestStatus>,
}

impl RequestSummaryBuilder {
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

    /// Consumes the builder and constructs a [`RequestSummary`].
    /// This method will fail if any of the following fields are not set:
    /// - [`request_id`](RequestSummaryBuilder::request_id)
    /// - [`model`](RequestSummaryBuilder::model)
    /// - [`status`](RequestSummaryBuilder::status)
    pub fn build(self) -> Result<RequestSummary, BuildError> {
        Ok(RequestSummary {
            request_id: self.request_id.ok_or_else(|| BuildError::missing_field("request_id"))?,
            model: self.model.ok_or_else(|| BuildError::missing_field("model"))?,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
        })
    }
}
