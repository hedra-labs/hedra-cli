pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Why an automatic top-up did not add funds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TopOffFailureReason {
    #[serde(rename = "payment_method_declined")]
    PaymentMethodDeclined,
}
impl fmt::Display for TopOffFailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::PaymentMethodDeclined => "payment_method_declined",
        };
        write!(f, "{}", s)
    }
}
