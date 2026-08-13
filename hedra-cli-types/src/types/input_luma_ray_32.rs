pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `luma-ray-32`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InputLumaRay32 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 6000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputLumaRay32AspectRatio,
    /// Output resolution.
    pub resolution: InputLumaRay32Resolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
}

impl InputLumaRay32 {
    pub fn builder() -> InputLumaRay32Builder {
        <InputLumaRay32Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputLumaRay32Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputLumaRay32AspectRatio>,
    resolution: Option<InputLumaRay32Resolution>,
    duration_ms: Option<i64>,
}

impl InputLumaRay32Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputLumaRay32AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputLumaRay32Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputLumaRay32`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputLumaRay32Builder::prompt)
    /// - [`aspect_ratio`](InputLumaRay32Builder::aspect_ratio)
    /// - [`resolution`](InputLumaRay32Builder::resolution)
    /// - [`duration_ms`](InputLumaRay32Builder::duration_ms)
    pub fn build(self) -> Result<InputLumaRay32, BuildError> {
        Ok(InputLumaRay32 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
        })
    }
}
