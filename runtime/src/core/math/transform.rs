use serde::{Deserialize, Serialize};

use crate::core::math::{matrix4::Matrix4x4, quaternion::Quaternion, vector3::Vector3};



#[derive(Clone, Serialize, Deserialize)]
pub struct Transform {
    m_position: Vector3,
    m_scale: Vector3,
    m_rotation: Quaternion,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            m_position: Vector3::new(0.0, 0.0, 0.0),
            m_scale: Vector3::new(1.0, 1.0, 1.0),
            m_rotation: Quaternion::identity(),
        }
    }
}

impl Transform {
    pub fn get_matrix(&self) -> Matrix4x4 {
        let translation_matrix = self.m_position.to_translate_matrix();
        let rotation_matrix = self.m_rotation.to_rotation_matrix();
        let scale_matrix = self.m_scale.to_scale_matrix();
        translation_matrix * rotation_matrix * scale_matrix
    }
}