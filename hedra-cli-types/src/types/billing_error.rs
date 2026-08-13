pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Why a request was refused for funds, and what would clear it.
/// 
/// The envelope's structured detail for `INSUFFICIENT_BALANCE`, the way
/// `details` is its structured detail for a validation failure. Every amount is
/// in `currency`, and `balance` is the same number `GET /balance` returns — so
/// a client can decide whether to top up, wait, or fail over without a second
/// round trip to work out which balance was short.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BillingError {
    /// Spendable balance on the account this request bills, at the moment it was refused. The same value `GET /balance` returns.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub balance: f64,
    /// What this request needed. Null when the refusal is a funding precondition rather than a priced one — an upload is not priced, and some models cannot be quoted until their inputs are measured (`POST /models/{model}/estimate` says so explicitly).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<f64>,
    /// ISO-4217 code the amounts above are denominated in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Where a human can add funds to the account this request bills. The API itself cannot add them.
    #[serde(default)]
    pub funding_url: String,
}

impl BillingError {
    pub fn builder() -> BillingErrorBuilder {
        <BillingErrorBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct BillingErrorBuilder {
    balance: Option<f64>,
    required: Option<f64>,
    currency: Option<String>,
    funding_url: Option<String>,
}

impl BillingErrorBuilder {
    pub fn balance(mut self, value: f64) -> Self {
        self.balance = Some(value);
        self
    }

    pub fn required(mut self, value: f64) -> Self {
        self.required = Some(value);
        self
    }

    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    pub fn funding_url(mut self, value: impl Into<String>) -> Self {
        self.funding_url = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`BillingError`].
    /// This method will fail if any of the following fields are not set:
    /// - [`balance`](BillingErrorBuilder::balance)
    /// - [`funding_url`](BillingErrorBuilder::funding_url)
    pub fn build(self) -> Result<BillingError, BuildError> {
        Ok(BillingError {
            balance: self.balance.ok_or_else(|| BuildError::missing_field("balance"))?,
            required: self.required,
            currency: self.currency,
            funding_url: self.funding_url.ok_or_else(|| BuildError::missing_field("funding_url"))?,
        })
    }
}
