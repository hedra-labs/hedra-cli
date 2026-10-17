pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One movement of the API wallet's balance.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TransactionRecord {
    /// The transaction's id. Stable, and the same id the V2 billing history reports for this transaction. An `llm_usage` transaction's id starts with `llm_usage:` and names its daily sum; a `sandbox_usage` transaction's id starts with `sandbox_usage:`.
    #[serde(default)]
    pub id: String,
    /// What moved the balance: `purchase` when funds were bought, `grant` when they were granted by a plan entitlement, `usage` when a job was charged, `llm_usage` for a day's chat requests to one model, summed into one row, `sandbox_usage` for what one GPU sandbox was charged, summed into one row, `refund` when a charge was returned, `adjustment` when Hedra corrected the balance, and `other` for a movement this API version does not yet name. The list is open and may gain values, so switch on it with a default branch; `amount` is authoritative for a kind you do not recognize.
    #[serde(default)]
    pub kind: String,
    /// The change to the balance, signed: negative for a charge, positive for funds arriving. Null for a transaction from before the wallet recorded amounts, whose movement is unknown rather than zero. For `llm_usage`, the sum of the day's charges so far; for `sandbox_usage`, the net of the sandbox's charges and refunds so far.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub amount: Option<f64>,
    /// ISO-4217 currency code for `amount`; null exactly when `amount` is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// ISO-8601 instant the balance moved. For `llm_usage`, the start of the day it covers; for `sandbox_usage`, when the sandbox was created.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    /// Set exactly when `kind` is `llm_usage`; null on every other row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_usage: Option<TransactionLlmUsage>,
    /// Set exactly when `kind` is `sandbox_usage`; null on every other row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_usage: Option<TransactionSandboxUsage>,
    /// What `amount` would have been at the model's list price, before the discount negotiated for the workspace, in `currency`; negative, like the debit it describes, even when a 100% discount made `amount` 0. Set only on a `usage` or `sandbox_usage` row a discount applied to; null on every other row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_amount: Option<f64>,
    /// The negotiated percentage off the model's list price that `amount` reflects, e.g. 20 or 12.5. Set exactly when `list_amount` is; null otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<f64>,
}

impl TransactionRecord {
    pub fn builder() -> TransactionRecordBuilder {
        <TransactionRecordBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct TransactionRecordBuilder {
    id: Option<String>,
    kind: Option<String>,
    amount: Option<f64>,
    currency: Option<String>,
    created_at: Option<DateTime<FixedOffset>>,
    llm_usage: Option<TransactionLlmUsage>,
    sandbox_usage: Option<TransactionSandboxUsage>,
    list_amount: Option<f64>,
    discount_percent: Option<f64>,
}

impl TransactionRecordBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn kind(mut self, value: impl Into<String>) -> Self {
        self.kind = Some(value.into());
        self
    }

    pub fn amount(mut self, value: f64) -> Self {
        self.amount = Some(value);
        self
    }

    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn llm_usage(mut self, value: TransactionLlmUsage) -> Self {
        self.llm_usage = Some(value);
        self
    }

    pub fn sandbox_usage(mut self, value: TransactionSandboxUsage) -> Self {
        self.sandbox_usage = Some(value);
        self
    }

    pub fn list_amount(mut self, value: f64) -> Self {
        self.list_amount = Some(value);
        self
    }

    pub fn discount_percent(mut self, value: f64) -> Self {
        self.discount_percent = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`TransactionRecord`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](TransactionRecordBuilder::id)
    /// - [`kind`](TransactionRecordBuilder::kind)
    /// - [`created_at`](TransactionRecordBuilder::created_at)
    pub fn build(self) -> Result<TransactionRecord, BuildError> {
        Ok(TransactionRecord {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            kind: self.kind.ok_or_else(|| BuildError::missing_field("kind"))?,
            amount: self.amount,
            currency: self.currency,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            llm_usage: self.llm_usage,
            sandbox_usage: self.sandbox_usage,
            list_amount: self.list_amount,
            discount_percent: self.discount_percent,
        })
    }
}
