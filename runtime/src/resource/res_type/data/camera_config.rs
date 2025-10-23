use crate::core::math::{vector2::Vector2, vector3::Vector3};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CameraPose {
    pub position: Vector3,
    pub target: Vector3,
    pub up: Vector3
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CameraConfig {
    pub pose: CameraPose,
    pub aspect: Vector2,
    pub z_far: f32,
    pub z_near: f32,
}