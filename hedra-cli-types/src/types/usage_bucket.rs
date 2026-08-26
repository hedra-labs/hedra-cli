pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One usage rollup row. `key` is ``"total"`` for the summary, an ISO date
/// (``YYYY-MM-DD``, UTC) for ``group_by=day``, or a public model id for
/// ``group_by=model``.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct UsageBucket {
    /// What this bucket rolls up: `"total"`, an ISO date (`YYYY-MM-DD`, UTC), or a public model id — per `group_by`.
    #[serde(default)]
    pub key: String,
    /// Jobs submitted in this bucket.
    #[serde(default)]
    pub jobs: i64,
    /// Settled LLM chat requests in this bucket. Unlike `jobs` (which counts submits, charged or not), this counts requests whose usage settled — a request refused before any work never appears, and a late settlement lands in the window the request was created in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<i64>,
    /// Net amount spent in this bucket.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub spent: f64,
}

impl UsageBucket {
    pub fn builder() -> UsageBucketBuilder {
        <UsageBucketBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct UsageBucketBuilder {
    key: Option<String>,
    jobs: Option<i64>,
    requests: Option<i64>,
    spent: Option<f64>,
}

impl UsageBucketBuilder {
    pub fn key(mut self, value: impl Into<String>) -> Self {
        self.key = Some(value.into());
        self
    }

    pub fn jobs(mut self, value: i64) -> Self {
        self.jobs = Some(value);
        self
    }

    pub fn requests(mut self, value: i64) -> Self {
        self.requests = Some(value);
        self
    }

    pub fn spent(mut self, value: f64) -> Self {
        self.spent = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`UsageBucket`].
    /// This method will fail if any of the following fields are not set:
    /// - [`key`](UsageBucketBuilder::key)
    /// - [`jobs`](UsageBucketBuilder::jobs)
    /// - [`spent`](UsageBucketBuilder::spent)
    pub fn build(self) -> Result<UsageBucket, BuildError> {
        Ok(UsageBucket {
            key: self.key.ok_or_else(|| BuildError::missing_field("key"))?,
            jobs: self.jobs.ok_or_else(|| BuildError::missing_field("jobs"))?,
            requests: self.requests,
            spent: self.spent.ok_or_else(|| BuildError::missing_field("spent"))?,
        })
    }
}
