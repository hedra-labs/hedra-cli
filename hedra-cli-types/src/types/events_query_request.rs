pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for events
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct EventsQueryRequest {
    /// The instance's replica number, from 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replica: Option<i64>,
}

impl EventsQueryRequest {
    pub fn builder() -> EventsQueryRequestBuilder {
        <EventsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct EventsQueryRequestBuilder {
    replica: Option<i64>,
}

impl EventsQueryRequestBuilder {
    pub fn replica(mut self, value: i64) -> Self {
        self.replica = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`EventsQueryRequest`].
    pub fn build(self) -> Result<EventsQueryRequest, BuildError> {
        Ok(EventsQueryRequest {
            replica: self.replica,
        })
    }
}

