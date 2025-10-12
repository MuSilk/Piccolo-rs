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

impl Transform {
    pub fn get_matrix(&self) -> nalgebra_glm::Mat4 {
        let translation_matrix = nalgebra_glm::translation(&self.m_position);
        let rotation_matrix = nalgebra_glm::quat_to_mat4(&self.m_rotation);
        let scale_matrix = nalgebra_glm::scaling(&self.m_scale);
        translation_matrix * rotation_matrix * scale_matrix
    }
}