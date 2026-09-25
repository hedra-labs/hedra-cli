pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Body of a `billing.top_off_failed` webhook.
/// 
/// Hedra sends this event when Stripe declines the payment method for an
/// automatic top-up of the API wallet. After a decline, Hedra holds further
/// automatic top-ups until the hold expires, the payment method changes or a
/// top-up is paid, and sends no event for a decline while a hold is in place.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopOffFailedEvent {
    /// The event's id, the same value as `X-Hedra-Webhook-Id`.
    #[serde(default)]
    pub id: String,
    /// Always `billing.top_off_failed`.
    pub r#type: BillingEventType,
    /// When the top-up was declined.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    pub reason: TopOffFailureReason,
    /// The amount the top-up tried to add.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub amount_attempted: f64,
    /// The API wallet's balance when this event was sent.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub balance: f64,
    /// ISO-4217 currency code for every amount in this event.
    #[serde(default)]
    pub currency: String,
}

impl TopOffFailedEvent {
    pub fn builder() -> TopOffFailedEventBuilder {
        <TopOffFailedEventBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct TopOffFailedEventBuilder {
    id: Option<String>,
    r#type: Option<BillingEventType>,
    created_at: Option<DateTime<FixedOffset>>,
    reason: Option<TopOffFailureReason>,
    amount_attempted: Option<f64>,
    balance: Option<f64>,
    currency: Option<String>,
}

impl TopOffFailedEventBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: BillingEventType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn reason(mut self, value: TopOffFailureReason) -> Self {
        self.reason = Some(value);
        self
    }

    pub fn amount_attempted(mut self, value: f64) -> Self {
        self.amount_attempted = Some(value);
        self
    }

    pub fn balance(mut self, value: f64) -> Self {
        self.balance = Some(value);
        self
    }

    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`TopOffFailedEvent`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](TopOffFailedEventBuilder::id)
    /// - [`r#type`](TopOffFailedEventBuilder::r#type)
    /// - [`created_at`](TopOffFailedEventBuilder::created_at)
    /// - [`reason`](TopOffFailedEventBuilder::reason)
    /// - [`amount_attempted`](TopOffFailedEventBuilder::amount_attempted)
    /// - [`balance`](TopOffFailedEventBuilder::balance)
    /// - [`currency`](TopOffFailedEventBuilder::currency)
    pub fn build(self) -> Result<TopOffFailedEvent, BuildError> {
        Ok(TopOffFailedEvent {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            reason: self.reason.ok_or_else(|| BuildError::missing_field("reason"))?,
            amount_attempted: self.amount_attempted.ok_or_else(|| BuildError::missing_field("amount_attempted"))?,
            balance: self.balance.ok_or_else(|| BuildError::missing_field("balance"))?,
            currency: self.currency.ok_or_else(|| BuildError::missing_field("currency"))?,
        })
    }
}
