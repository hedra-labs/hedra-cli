pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct StatusLog {
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub message: String,
}

impl StatusLog {
    pub fn builder() -> StatusLogBuilder {
        <StatusLogBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct StatusLogBuilder {
    timestamp: Option<String>,
    message: Option<String>,
}

impl StatusLogBuilder {
    pub fn timestamp(mut self, value: impl Into<String>) -> Self {
        self.timestamp = Some(value.into());
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`StatusLog`].
    /// This method will fail if any of the following fields are not set:
    /// - [`timestamp`](StatusLogBuilder::timestamp)
    /// - [`message`](StatusLogBuilder::message)
    pub fn build(self) -> Result<StatusLog, BuildError> {
        Ok(StatusLog {
            timestamp: self.timestamp.ok_or_else(|| BuildError::missing_field("timestamp"))?,
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
        })
    }
}
