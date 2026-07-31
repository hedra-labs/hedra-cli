pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WebhookJobFailedPayloadXHedraWebhookEvent {
    #[serde(rename = "job.failed")]
    JobFailed,
}
impl fmt::Display for WebhookJobFailedPayloadXHedraWebhookEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::JobFailed => "job.failed",
        };
        write!(f, "{}", s)
    }
}
