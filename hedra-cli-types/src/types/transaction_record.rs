pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One movement of the API wallet's balance.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TransactionRecord {
    /// The transaction's id. Stable, and the same id the V2 billing history reports for this row.
    #[serde(default)]
    pub id: String,
    /// What moved the balance: `purchase` when funds were bought, `grant` when they were granted by a plan entitlement, `usage` when a job was charged, `refund` when a charge was returned, `adjustment` when Hedra corrected the balance, and `other` for a movement this API version does not yet name. The list is open and may gain values, so switch on it with a default branch; `amount` is authoritative for a kind you do not recognize.
    #[serde(default)]
    pub kind: String,
    /// The change to the balance, signed: negative for a charge, positive for funds arriving. Null for a row written before the wallet recorded amounts, whose movement is unknown rather than zero; no such row exists in production.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub amount: Option<f64>,
    /// ISO-4217 currency code for `amount`; null exactly when `amount` is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// ISO-8601 instant the balance moved.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
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
        })
    }
}
