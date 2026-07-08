pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Metrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_time_ms: Option<i64>,
}

impl Metrics {
    pub fn builder() -> MetricsBuilder {
        <MetricsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct MetricsBuilder {
    inference_time_ms: Option<i64>,
}

impl MetricsBuilder {
    pub fn inference_time_ms(mut self, value: i64) -> Self {
        self.inference_time_ms = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`Metrics`].
    pub fn build(self) -> Result<Metrics, BuildError> {
        Ok(Metrics {
            inference_time_ms: self.inference_time_ms,
        })
    }
}
