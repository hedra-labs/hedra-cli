pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WebhookJobCompletedPayloadXHedraWebhookEvent {
    #[serde(rename = "job.completed")]
    JobCompleted,
}
impl fmt::Display for WebhookJobCompletedPayloadXHedraWebhookEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::JobCompleted => "job.completed",
        };
        write!(f, "{}", s)
    }
}
