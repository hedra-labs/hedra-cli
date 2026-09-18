pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One camera pose at a normalized point in the generated clip.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputMinimaxH3MaxCameraControlsCameraTrajectoryItem {
    /// When this pose applies, from 0 at the start to 1 at the end.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub time: f64,
    /// Horizontal camera angle around the subject, in degrees.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub azimuth: f64,
    /// Vertical camera angle around the subject, in degrees.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub elevation: f64,
    /// Camera distance from the subject, in normalized scene units.
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers")]
    pub distance: f64,
}

impl InputMinimaxH3MaxCameraControlsCameraTrajectoryItem {
    pub fn builder() -> InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder {
        <InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder {
    time: Option<f64>,
    azimuth: Option<f64>,
    elevation: Option<f64>,
    distance: Option<f64>,
}

impl InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder {
    pub fn time(mut self, value: f64) -> Self {
        self.time = Some(value);
        self
    }

    pub fn azimuth(mut self, value: f64) -> Self {
        self.azimuth = Some(value);
        self
    }

    pub fn elevation(mut self, value: f64) -> Self {
        self.elevation = Some(value);
        self
    }

    pub fn distance(mut self, value: f64) -> Self {
        self.distance = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMinimaxH3MaxCameraControlsCameraTrajectoryItem`].
    /// This method will fail if any of the following fields are not set:
    /// - [`time`](InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder::time)
    /// - [`azimuth`](InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder::azimuth)
    /// - [`elevation`](InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder::elevation)
    /// - [`distance`](InputMinimaxH3MaxCameraControlsCameraTrajectoryItemBuilder::distance)
    pub fn build(self) -> Result<InputMinimaxH3MaxCameraControlsCameraTrajectoryItem, BuildError> {
        Ok(InputMinimaxH3MaxCameraControlsCameraTrajectoryItem {
            time: self.time.ok_or_else(|| BuildError::missing_field("time"))?,
            azimuth: self.azimuth.ok_or_else(|| BuildError::missing_field("azimuth"))?,
            elevation: self.elevation.ok_or_else(|| BuildError::missing_field("elevation"))?,
            distance: self.distance.ok_or_else(|| BuildError::missing_field("distance"))?,
        })
    }
}
