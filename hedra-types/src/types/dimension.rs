pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Dimension {
    /// Width of the image.
    #[serde(default)]
    pub width: i64,
    /// Height of the image.
    #[serde(default)]
    pub height: i64,
}

impl Dimension {
    pub fn builder() -> DimensionBuilder {
        <DimensionBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct DimensionBuilder {
    width: Option<i64>,
    height: Option<i64>,
}

impl DimensionBuilder {
    pub fn width(mut self, value: i64) -> Self {
        self.width = Some(value);
        self
    }

    pub fn height(mut self, value: i64) -> Self {
        self.height = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`Dimension`].
    /// This method will fail if any of the following fields are not set:
    /// - [`width`](DimensionBuilder::width)
    /// - [`height`](DimensionBuilder::height)
    pub fn build(self) -> Result<Dimension, BuildError> {
        Ok(Dimension {
            width: self.width.ok_or_else(|| BuildError::missing_field("width"))?,
            height: self.height.ok_or_else(|| BuildError::missing_field("height"))?,
        })
    }
}
