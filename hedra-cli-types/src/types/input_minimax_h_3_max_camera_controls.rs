pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `minimax-h3-max-camera-controls`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMinimaxH3MaxCameraControls {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. From 1 to 7000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    pub resolution: InputMinimaxH3MaxCameraControlsResolution,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame. From 256px to 5760px on each side, with an aspect ratio from 0.4 to 2.5, and at most 30 MB.
    pub start_image: InputMinimaxH3MaxCameraControlsStartImage,
    /// Camera path through the scene, as poses ordered by strictly increasing `time`. The first pose holds until its `time` and the last holds to the end of the clip; the camera interpolates between them. `time` runs 0 (start) to 1 (end), `azimuth` is the horizontal angle around the subject in degrees, `elevation` the vertical angle in degrees from -90 to 90, and `distance` the distance from the subject in scene units (above 0; 1 is where the input frame sits). Consecutive azimuths may travel at most 32 full turns in total. 2 to 12 items.
    #[serde(default)]
    pub camera_trajectory: Vec<InputMinimaxH3MaxCameraControlsCameraTrajectoryItem>,
}

impl InputMinimaxH3MaxCameraControls {
    pub fn builder() -> InputMinimaxH3MaxCameraControlsBuilder {
        <InputMinimaxH3MaxCameraControlsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputMinimaxH3MaxCameraControlsBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputMinimaxH3MaxCameraControlsResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputMinimaxH3MaxCameraControlsStartImage>,
    camera_trajectory: Option<Vec<InputMinimaxH3MaxCameraControlsCameraTrajectoryItem>>,
}

impl InputMinimaxH3MaxCameraControlsBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputMinimaxH3MaxCameraControlsResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputMinimaxH3MaxCameraControlsStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn camera_trajectory(mut self, value: Vec<InputMinimaxH3MaxCameraControlsCameraTrajectoryItem>) -> Self {
        self.camera_trajectory = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputMinimaxH3MaxCameraControls`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputMinimaxH3MaxCameraControlsBuilder::prompt)
    /// - [`resolution`](InputMinimaxH3MaxCameraControlsBuilder::resolution)
    /// - [`duration_ms`](InputMinimaxH3MaxCameraControlsBuilder::duration_ms)
    /// - [`start_image`](InputMinimaxH3MaxCameraControlsBuilder::start_image)
    /// - [`camera_trajectory`](InputMinimaxH3MaxCameraControlsBuilder::camera_trajectory)
    pub fn build(self) -> Result<InputMinimaxH3MaxCameraControls, BuildError> {
        Ok(InputMinimaxH3MaxCameraControls {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            camera_trajectory: self.camera_trajectory.ok_or_else(|| BuildError::missing_field("camera_trajectory"))?,
        })
    }
}
