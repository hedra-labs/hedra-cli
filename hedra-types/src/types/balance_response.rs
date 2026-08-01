pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Spendable balance on the account this key bills.
/// 
/// Sourced from the workspace wallet for a workspace-bound key, else the
/// caller's personal balance. While the segregated API wallet is enabled, both
/// key types instead read the API wallet that generation debits draw on (a
/// personal key resolves its bound workspace).
/// 
/// Every amount is in US dollars.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BalanceResponse {
    /// Spendable balance.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub balance: f64,
    /// Amount consumed in the current billing period; null for the non-expiring API wallet (see GET /v3/usage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spent: Option<f64>,
    /// ISO-4217 currency code for every amount in this response.
    #[serde(default)]
    pub currency: String,
}

impl BalanceResponse {
    pub fn builder() -> BalanceResponseBuilder {
        <BalanceResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct BalanceResponseBuilder {
    balance: Option<f64>,
    spent: Option<f64>,
    currency: Option<String>,
}

impl BalanceResponseBuilder {
    pub fn balance(mut self, value: f64) -> Self {
        self.balance = Some(value);
        self
    }

    pub fn spent(mut self, value: f64) -> Self {
        self.spent = Some(value);
        self
    }

    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`BalanceResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`balance`](BalanceResponseBuilder::balance)
    /// - [`currency`](BalanceResponseBuilder::currency)
    pub fn build(self) -> Result<BalanceResponse, BuildError> {
        Ok(BalanceResponse {
            balance: self.balance.ok_or_else(|| BuildError::missing_field("balance"))?,
            spent: self.spent,
            currency: self.currency.ok_or_else(|| BuildError::missing_field("currency"))?,
        })
    }
}
