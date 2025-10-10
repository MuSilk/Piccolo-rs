use nalgebra_glm::{Vec3, Quat};

#[derive(Clone)]
pub struct Transform {
    m_position: Vec3,
    m_scale: Vec3,
    m_rotation: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            m_position: Vec3::new(0.0, 0.0, 0.0),
            m_scale: Vec3::new(1.0, 1.0, 1.0),
            m_rotation: Quat::identity(),
        }
    }
}