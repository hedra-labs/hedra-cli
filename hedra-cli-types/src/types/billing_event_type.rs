pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The billing events an outbound webhook announces.
/// 
/// Separate from `WebhookEventType`, which names the events of the job
/// deliveries `GET /webhooks/deliveries` lists. `GET /webhooks/deliveries`
/// does not list these.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BillingEventType {
    #[serde(rename = "billing.top_off_failed")]
    BillingTopOffFailed,
}
impl fmt::Display for BillingEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::BillingTopOffFailed => "billing.top_off_failed",
        };
        write!(f, "{}", s)
    }
}
