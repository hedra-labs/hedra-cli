pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JobsStreamResponse {
        StatusResponse(StatusResponse),

        JobLogItem(JobLogItem),
}

impl JobsStreamResponse {
    pub fn is_status_response(&self) -> bool {
        matches!(self, Self::StatusResponse(_))
    }

    pub fn is_job_log_item(&self) -> bool {
        matches!(self, Self::JobLogItem(_))
    }


    pub fn as_status_response(&self) -> Option<&StatusResponse> {
        match self {
                    Self::StatusResponse(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_status_response(self) -> Option<StatusResponse> {
        match self {
                    Self::StatusResponse(value) => Some(value),
                    _ => None,
                }
    }

    pub fn as_job_log_item(&self) -> Option<&JobLogItem> {
        match self {
                    Self::JobLogItem(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_job_log_item(self) -> Option<JobLogItem> {
        match self {
                    Self::JobLogItem(value) => Some(value),
                    _ => None,
                }
    }
}

impl fmt::Display for JobsStreamResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StatusResponse(value) => write!(f, "{}", serde_json::to_string(value).unwrap_or_else(|_| format!("{:?}", value))),
            Self::JobLogItem(value) => write!(f, "{}", serde_json::to_string(value).unwrap_or_else(|_| format!("{:?}", value))),
        }
    }
}
