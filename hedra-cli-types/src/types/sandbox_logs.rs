pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// An instance's newest log output.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxLogs {
    /// The output, newest last, as UTF-8; a byte sequence that is not UTF-8 reads as U+FFFD.
    #[serde(default)]
    pub content: String,
    /// Whether older output was cut, so `content` starts partway through the instance's output.
    #[serde(default)]
    pub truncated: bool,
}

impl SandboxLogs {
    pub fn builder() -> SandboxLogsBuilder {
        <SandboxLogsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxLogsBuilder {
    content: Option<String>,
    truncated: Option<bool>,
}

impl SandboxLogsBuilder {
    pub fn content(mut self, value: impl Into<String>) -> Self {
        self.content = Some(value.into());
        self
    }

    pub fn truncated(mut self, value: bool) -> Self {
        self.truncated = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxLogs`].
    /// This method will fail if any of the following fields are not set:
    /// - [`content`](SandboxLogsBuilder::content)
    /// - [`truncated`](SandboxLogsBuilder::truncated)
    pub fn build(self) -> Result<SandboxLogs, BuildError> {
        Ok(SandboxLogs {
            content: self.content.ok_or_else(|| BuildError::missing_field("content"))?,
            truncated: self.truncated.ok_or_else(|| BuildError::missing_field("truncated"))?,
        })
    }
}
