pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// What a sandbox costs, has paid and was charged.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SandboxBillingSummary {
    /// The sandbox's rate per hour for all its instances, after your workspace's discount.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub usd_per_hour: f64,
    /// What the sandbox has paid from the API wallet, net of refunds.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub paid_usd: f64,
    /// The charge once settled. Before that, on `GET /sandboxes/{sandbox}` only, the charge so far; null on the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub charged_usd: Option<f64>,
    /// Whether the charge is final and the difference from `paid_usd` has been refunded or debited.
    #[serde(default)]
    pub settled: bool,
}

impl SandboxBillingSummary {
    pub fn builder() -> SandboxBillingSummaryBuilder {
        <SandboxBillingSummaryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxBillingSummaryBuilder {
    usd_per_hour: Option<f64>,
    paid_usd: Option<f64>,
    charged_usd: Option<f64>,
    settled: Option<bool>,
}

impl SandboxBillingSummaryBuilder {
    pub fn usd_per_hour(mut self, value: f64) -> Self {
        self.usd_per_hour = Some(value);
        self
    }

    pub fn paid_usd(mut self, value: f64) -> Self {
        self.paid_usd = Some(value);
        self
    }

    pub fn charged_usd(mut self, value: f64) -> Self {
        self.charged_usd = Some(value);
        self
    }

    pub fn settled(mut self, value: bool) -> Self {
        self.settled = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxBillingSummary`].
    /// This method will fail if any of the following fields are not set:
    /// - [`usd_per_hour`](SandboxBillingSummaryBuilder::usd_per_hour)
    /// - [`paid_usd`](SandboxBillingSummaryBuilder::paid_usd)
    /// - [`settled`](SandboxBillingSummaryBuilder::settled)
    pub fn build(self) -> Result<SandboxBillingSummary, BuildError> {
        Ok(SandboxBillingSummary {
            usd_per_hour: self.usd_per_hour.ok_or_else(|| BuildError::missing_field("usd_per_hour"))?,
            paid_usd: self.paid_usd.ok_or_else(|| BuildError::missing_field("paid_usd"))?,
            charged_usd: self.charged_usd,
            settled: self.settled.ok_or_else(|| BuildError::missing_field("settled"))?,
        })
    }
}
