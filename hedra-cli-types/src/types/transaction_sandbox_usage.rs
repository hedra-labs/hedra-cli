pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The GPU sandbox a `sandbox_usage` transaction sums.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct TransactionSandboxUsage {
    /// The GPU model and GPU count per instance, such as `h100:1`.
    #[serde(default)]
    pub accelerator: String,
    /// How many instances the sandbox ran.
    #[serde(default)]
    pub replicas: i64,
    /// When the sandbox was created. The transaction's `created_at` is this instant.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    /// When the sandbox's last instance stopped, or null while it runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub ended_at: Option<DateTime<FixedOffset>>,
}

impl TransactionSandboxUsage {
    pub fn builder() -> TransactionSandboxUsageBuilder {
        <TransactionSandboxUsageBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct TransactionSandboxUsageBuilder {
    accelerator: Option<String>,
    replicas: Option<i64>,
    created_at: Option<DateTime<FixedOffset>>,
    ended_at: Option<DateTime<FixedOffset>>,
}

impl TransactionSandboxUsageBuilder {
    pub fn accelerator(mut self, value: impl Into<String>) -> Self {
        self.accelerator = Some(value.into());
        self
    }

    pub fn replicas(mut self, value: i64) -> Self {
        self.replicas = Some(value);
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn ended_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.ended_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`TransactionSandboxUsage`].
    /// This method will fail if any of the following fields are not set:
    /// - [`accelerator`](TransactionSandboxUsageBuilder::accelerator)
    /// - [`replicas`](TransactionSandboxUsageBuilder::replicas)
    /// - [`created_at`](TransactionSandboxUsageBuilder::created_at)
    pub fn build(self) -> Result<TransactionSandboxUsage, BuildError> {
        Ok(TransactionSandboxUsage {
            accelerator: self.accelerator.ok_or_else(|| BuildError::missing_field("accelerator"))?,
            replicas: self.replicas.ok_or_else(|| BuildError::missing_field("replicas"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            ended_at: self.ended_at,
        })
    }
}
