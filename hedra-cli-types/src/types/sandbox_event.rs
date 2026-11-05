pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One lifecycle event of an instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SandboxEvent {
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub at: DateTime<FixedOffset>,
    pub kind: SandboxEventKind,
    /// What happened, for a person to read.
    #[serde(default)]
    pub message: String,
}

impl SandboxEvent {
    pub fn builder() -> SandboxEventBuilder {
        <SandboxEventBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxEventBuilder {
    at: Option<DateTime<FixedOffset>>,
    kind: Option<SandboxEventKind>,
    message: Option<String>,
}

impl SandboxEventBuilder {
    pub fn at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.at = Some(value);
        self
    }

    pub fn kind(mut self, value: SandboxEventKind) -> Self {
        self.kind = Some(value);
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SandboxEvent`].
    /// This method will fail if any of the following fields are not set:
    /// - [`at`](SandboxEventBuilder::at)
    /// - [`kind`](SandboxEventBuilder::kind)
    /// - [`message`](SandboxEventBuilder::message)
    pub fn build(self) -> Result<SandboxEvent, BuildError> {
        Ok(SandboxEvent {
            at: self.at.ok_or_else(|| BuildError::missing_field("at"))?,
            kind: self.kind.ok_or_else(|| BuildError::missing_field("kind"))?,
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
        })
    }
}
