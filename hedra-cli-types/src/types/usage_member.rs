pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// What one member of the workspace spent in the window.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageMember {
    /// The member: the person who submitted the jobs and created the sandboxes counted here, or the workspace's service key. A member removed from the workspace keeps their entry for what they spent while they were in it. A member whose account was deleted no longer has their jobs counted, as no v3 read returns a deleted account's jobs; their sandboxes still are, in one entry whose `email` and `name` are null, shared by every deleted account.
    pub created_by: SandboxCreator,
    /// Jobs the member submitted in the window, chat completions included.
    #[serde(default)]
    pub jobs: i64,
    /// Net amount the member spent in the window.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub spent: f64,
}

impl UsageMember {
    pub fn builder() -> UsageMemberBuilder {
        <UsageMemberBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct UsageMemberBuilder {
    created_by: Option<SandboxCreator>,
    jobs: Option<i64>,
    spent: Option<f64>,
}

impl UsageMemberBuilder {
    pub fn created_by(mut self, value: SandboxCreator) -> Self {
        self.created_by = Some(value);
        self
    }

    pub fn jobs(mut self, value: i64) -> Self {
        self.jobs = Some(value);
        self
    }

    pub fn spent(mut self, value: f64) -> Self {
        self.spent = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`UsageMember`].
    /// This method will fail if any of the following fields are not set:
    /// - [`created_by`](UsageMemberBuilder::created_by)
    /// - [`jobs`](UsageMemberBuilder::jobs)
    /// - [`spent`](UsageMemberBuilder::spent)
    pub fn build(self) -> Result<UsageMember, BuildError> {
        Ok(UsageMember {
            created_by: self.created_by.ok_or_else(|| BuildError::missing_field("created_by"))?,
            jobs: self.jobs.ok_or_else(|| BuildError::missing_field("jobs"))?,
            spent: self.spent.ok_or_else(|| BuildError::missing_field("spent"))?,
        })
    }
}
