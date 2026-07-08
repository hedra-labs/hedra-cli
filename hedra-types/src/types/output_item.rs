pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One per-modality output. `outputs` is always an array, even for one.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct OutputItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RequestStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,
}

impl OutputItem {
    pub fn builder() -> OutputItemBuilder {
        <OutputItemBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct OutputItemBuilder {
    status: Option<RequestStatus>,
    url: Option<String>,
    content_type: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    duration_ms: Option<i64>,
    fps: Option<i64>,
    error: Option<ErrorEnvelope>,
}

impl OutputItemBuilder {
    pub fn status(mut self, value: RequestStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn content_type(mut self, value: impl Into<String>) -> Self {
        self.content_type = Some(value.into());
        self
    }

    pub fn width(mut self, value: i64) -> Self {
        self.width = Some(value);
        self
    }

    pub fn height(mut self, value: i64) -> Self {
        self.height = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn fps(mut self, value: i64) -> Self {
        self.fps = Some(value);
        self
    }

    pub fn error(mut self, value: ErrorEnvelope) -> Self {
        self.error = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`OutputItem`].
    pub fn build(self) -> Result<OutputItem, BuildError> {
        Ok(OutputItem {
            status: self.status,
            url: self.url,
            content_type: self.content_type,
            width: self.width,
            height: self.height,
            duration_ms: self.duration_ms,
            fps: self.fps,
            error: self.error,
        })
    }
}
