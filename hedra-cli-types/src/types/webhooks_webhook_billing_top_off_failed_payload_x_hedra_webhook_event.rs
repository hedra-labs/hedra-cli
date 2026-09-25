pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WebhookBillingTopOffFailedPayloadXHedraWebhookEvent {
    #[serde(rename = "billing.top_off_failed")]
    BillingTopOffFailed,
}
impl fmt::Display for WebhookBillingTopOffFailedPayloadXHedraWebhookEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::BillingTopOffFailed => "billing.top_off_failed",
        };
        write!(f, "{}", s)
    }
}
