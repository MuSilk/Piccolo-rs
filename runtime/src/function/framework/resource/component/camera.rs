use crate::core::math::{quaternion::Quaternion, vector3::Vector3};

#[derive(Clone)]
pub enum CameraParameter {
    FirstPerson(FirstPersonCameraParameter),
    ThirdPerson(ThirdPersonCameraParameter),
    Free(FreeCameraParameter),
}

#[derive(Clone)]
pub struct FirstPersonCameraParameter {
    pub m_fov: f32,
    pub m_vertical_offset: f32,
    /// 相对角色原点（与碰撞体 AABB 最小角对齐）的水平偏移，取与碰撞 `half_extent` 相同的 xy 时，眼睛在水平面中心。
    pub m_horizontal_eye_offset: Vector3,
}

impl Default for FirstPersonCameraParameter {
    fn default() -> Self {
        FirstPersonCameraParameter {
            m_fov: 50.0,
            m_vertical_offset: 1.5,
            m_horizontal_eye_offset: Vector3::new(0.34, 0.34, 0.0),
        }
    }
}

#[derive(Clone)]
pub struct ThirdPersonCameraParameter {
    pub m_fov: f32, 
    pub m_horizontal_offset: f32,
    pub m_vertical_offset: f32,
    pub m_cursor_pitch: Quaternion,
    pub m_cursor_yaw: Quaternion,
}

impl Default for ThirdPersonCameraParameter {
    fn default() -> Self {
        ThirdPersonCameraParameter {
            m_fov: 50.0,
            m_horizontal_offset: 3.0,
            m_vertical_offset: 2.5,
            m_cursor_pitch: Quaternion::identity(),
            m_cursor_yaw: Quaternion::identity(),
        }
    }
}

#[derive(Clone)]
pub struct FreeCameraParameter {
    pub m_fov: f32, 
    pub m_speed: f32,
}

impl Default for FreeCameraParameter {
    fn default() -> Self {
        FreeCameraParameter {
            m_fov: 50.0,
            m_speed: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct CameraComponentRes {
    pub m_parameter: CameraParameter,
}

impl CameraComponentRes {
    pub fn get_fov(&self) -> f32 {
        match &self.m_parameter {
            CameraParameter::FirstPerson(param) => param.m_fov,
            CameraParameter::ThirdPerson(param) => param.m_fov, 
            CameraParameter::Free(param) => param.m_fov,
        }
    }
}