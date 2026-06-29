pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Information about the current balance of Credits and usage.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CreditBalance {
    /// Remaining credits not yet used.
    #[serde(default)]
    pub remaining: i64,
    /// Credits that will expire in the future.
    #[serde(default)]
    pub expiring: i64,
    /// Credits used in the current billing period.
    #[serde(default)]
    pub used: i64,
    /// Credits for each workspace mapped by workspace_id. Only included if user is in a workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_credits: Option<HashMap<String, Option<i64>>>,
}

impl CreditBalance {
    pub fn builder() -> CreditBalanceBuilder {
        <CreditBalanceBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct CreditBalanceBuilder {
    remaining: Option<i64>,
    expiring: Option<i64>,
    used: Option<i64>,
    workspace_credits: Option<HashMap<String, Option<i64>>>,
}

impl CreditBalanceBuilder {
    pub fn remaining(mut self, value: i64) -> Self {
        self.remaining = Some(value);
        self
    }

    pub fn expiring(mut self, value: i64) -> Self {
        self.expiring = Some(value);
        self
    }

    pub fn used(mut self, value: i64) -> Self {
        self.used = Some(value);
        self
    }

    pub fn workspace_credits(mut self, value: HashMap<String, Option<i64>>) -> Self {
        self.workspace_credits = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`CreditBalance`].
    /// This method will fail if any of the following fields are not set:
    /// - [`remaining`](CreditBalanceBuilder::remaining)
    /// - [`expiring`](CreditBalanceBuilder::expiring)
    /// - [`used`](CreditBalanceBuilder::used)
    pub fn build(self) -> Result<CreditBalance, BuildError> {
        Ok(CreditBalance {
            remaining: self.remaining.ok_or_else(|| BuildError::missing_field("remaining"))?,
            expiring: self.expiring.ok_or_else(|| BuildError::missing_field("expiring"))?,
            used: self.used.ok_or_else(|| BuildError::missing_field("used"))?,
            workspace_credits: self.workspace_credits,
        })
    }
}
