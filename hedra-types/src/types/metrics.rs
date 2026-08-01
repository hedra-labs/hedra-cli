pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Timing measured for a completed job.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Metrics {
    /// Wall-clock milliseconds between this job's `started` and `completed` lifecycle events. It brackets provider queueing, generation, output download, and any failed attempts with their retry backoff, so it measures the whole job rather than the model's own inference time, and it is not a provider-reported figure. Read from the job's durable lifecycle records, so polled results and webhook deliveries report the same value. Null when the job did not record both events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<i64>,
}

impl Metrics {
    pub fn builder() -> MetricsBuilder {
        <MetricsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct MetricsBuilder {
    processing_time_ms: Option<i64>,
}

impl MetricsBuilder {
    pub fn processing_time_ms(mut self, value: i64) -> Self {
        self.processing_time_ms = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`Metrics`].
    pub fn build(self) -> Result<Metrics, BuildError> {
        Ok(Metrics {
            processing_time_ms: self.processing_time_ms,
        })
    }
}
