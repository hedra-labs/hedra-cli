pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageResponse {
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub start: DateTime<FixedOffset>,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub end: DateTime<FixedOffset>,
    pub group_by: UsageGroupBy,
    #[serde(default)]
    pub total_jobs: i64,
    /// Net amount spent across the whole window.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub total_spent: f64,
    /// ISO-4217 currency code for every amount in this response.
    #[serde(default)]
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<UsageBucket>>,
}

impl UsageResponse {
    pub fn builder() -> UsageResponseBuilder {
        <UsageResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct UsageResponseBuilder {
    start: Option<DateTime<FixedOffset>>,
    end: Option<DateTime<FixedOffset>>,
    group_by: Option<UsageGroupBy>,
    total_jobs: Option<i64>,
    total_spent: Option<f64>,
    currency: Option<String>,
    data: Option<Vec<UsageBucket>>,
}

impl UsageResponseBuilder {
    pub fn start(mut self, value: DateTime<FixedOffset>) -> Self {
        self.start = Some(value);
        self
    }

    pub fn end(mut self, value: DateTime<FixedOffset>) -> Self {
        self.end = Some(value);
        self
    }

    pub fn group_by(mut self, value: UsageGroupBy) -> Self {
        self.group_by = Some(value);
        self
    }

    pub fn total_jobs(mut self, value: i64) -> Self {
        self.total_jobs = Some(value);
        self
    }

    pub fn total_spent(mut self, value: f64) -> Self {
        self.total_spent = Some(value);
        self
    }

    pub fn currency(mut self, value: impl Into<String>) -> Self {
        self.currency = Some(value.into());
        self
    }

    pub fn data(mut self, value: Vec<UsageBucket>) -> Self {
        self.data = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`UsageResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`start`](UsageResponseBuilder::start)
    /// - [`end`](UsageResponseBuilder::end)
    /// - [`group_by`](UsageResponseBuilder::group_by)
    /// - [`total_jobs`](UsageResponseBuilder::total_jobs)
    /// - [`total_spent`](UsageResponseBuilder::total_spent)
    /// - [`currency`](UsageResponseBuilder::currency)
    pub fn build(self) -> Result<UsageResponse, BuildError> {
        Ok(UsageResponse {
            start: self.start.ok_or_else(|| BuildError::missing_field("start"))?,
            end: self.end.ok_or_else(|| BuildError::missing_field("end"))?,
            group_by: self.group_by.ok_or_else(|| BuildError::missing_field("group_by"))?,
            total_jobs: self.total_jobs.ok_or_else(|| BuildError::missing_field("total_jobs"))?,
            total_spent: self.total_spent.ok_or_else(|| BuildError::missing_field("total_spent"))?,
            currency: self.currency.ok_or_else(|| BuildError::missing_field("currency"))?,
            data: self.data,
        })
    }
}
